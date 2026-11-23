// SPDX-License-Identifier: GPL-3.0-or-later
// The calendars Backlith reads: their colours and switches, adding a
// calendar link, and uploading schedule files. These changes take effect
// straight away; only layout changes wait for Apply.
import { useState } from "react";
import type { Api } from "../api";
import type { CalendarSource } from "../generated/CalendarSource";
import type { PlannerState } from "../generated/PlannerState";
import type { SourceStatus } from "../generated/SourceStatus";
import type { UploadError } from "../generated/UploadError";
import { copyText } from "./clipboard";
import { nextColor, PALETTE } from "./palette";
import { field, Note, primary, secondary, Section } from "./ui";

type Props = { api: Api; state: PlannerState; reload: () => Promise<void> };

const upcoming = (n: number) => (n === 1 ? "1 event in the next 8 weeks" : `${n} events in the next 8 weeks`);
const errorText = (e: unknown) => (typeof e === "string" ? e : e instanceof Error ? e.message : String(e));

function SourceRow({ source, status, api, reload }: { source: CalendarSource; status?: SourceStatus; api: Api; reload: () => Promise<void> }) {
  const [picking, setPicking] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const update = async (color: string, enabled: boolean) => {
    try {
      await api.updateSource(source.id, source.name, color, enabled);
      await reload();
    } catch (e) {
      setError(errorText(e));
    }
  };
  return (
    <li style={{ listStyle: "none", display: "flex", flexDirection: "column", gap: 6 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <button type="button" aria-label={`Colour for ${source.name}`} onClick={() => setPicking(!picking)} style={{ width: 28, height: 28, borderRadius: 8, border: "2px solid rgba(255,255,255,0.2)", background: source.color, cursor: "pointer", flex: "none" }} />
        <label style={{ display: "flex", alignItems: "center", gap: 8, flex: 1, minWidth: 0, fontSize: 14 }}>
          <input type="checkbox" aria-label={`Show ${source.name}`} checked={source.enabled} onChange={() => void update(source.color, !source.enabled)} style={{ width: 20, height: 20, flex: "none" }} />
          <span style={{ whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{source.name}</span>
        </label>
        {confirming ? (
          <span style={{ display: "flex", gap: 6 }}>
            <button type="button" style={{ ...secondary, minHeight: 32 }} onClick={() => void api.removeSource(source.id).then(reload, (e) => setError(errorText(e)))}>
              Remove
            </button>
            <button type="button" style={{ ...secondary, minHeight: 32 }} onClick={() => setConfirming(false)}>
              Keep
            </button>
          </span>
        ) : (
          <button type="button" aria-label={`Remove ${source.name}`} onClick={() => setConfirming(true)} style={{ ...secondary, minHeight: 32, padding: "0 10px" }}>
            ×
          </button>
        )}
      </div>
      {picking && (
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap", paddingLeft: 38 }}>
          {PALETTE.map((c) => (
            <button
              key={c}
              type="button"
              aria-label={`Colour ${c}`}
              onClick={() => {
                setPicking(false);
                void update(c, source.enabled);
              }}
              style={{ width: 28, height: 28, borderRadius: 8, border: c === source.color ? "2px solid #E6EBF2" : "2px solid transparent", background: c, cursor: "pointer" }}
            />
          ))}
        </div>
      )}
      <div style={{ paddingLeft: 38 }}>
        {status && !status.ok && status.message ? <Note tone="error">{status.message}</Note> : status ? <Note>{upcoming(status.eventCount)}</Note> : null}
        {error && <Note tone="error">{error}</Note>}
      </div>
    </li>
  );
}

function AddLinkForm({ api, state, reload }: Props) {
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<{ ok: boolean; text: string } | null>(null);
  const add = async () => {
    setBusy(true);
    setResult(null);
    try {
      const status = await api.addLink(name.trim() || "Calendar", nextColor(state.settings.calendars.map((c) => c.color)), url.trim());
      setResult({ ok: true, text: upcoming(status.eventCount) });
      setName("");
      setUrl("");
      await reload();
    } catch (e) {
      setResult({ ok: false, text: errorText(e) });
    } finally {
      setBusy(false);
    }
  };
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void add();
      }}
      style={{ display: "flex", flexDirection: "column", gap: 8 }}
    >
      <label htmlFor="link-name" style={{ fontSize: 14 }}>
        Name
      </label>
      <input id="link-name" value={name} onChange={(e) => setName(e.target.value)} placeholder="Work" style={field} />
      <label htmlFor="link-url" style={{ fontSize: 14 }}>
        Calendar link
      </label>
      <input id="link-url" value={url} onChange={(e) => setUrl(e.target.value)} placeholder="https://… or webcal://…" autoComplete="off" spellCheck={false} style={field} />
      <Note>Use your calendar's secret iCal address. It is kept in your system's password store, never in Backlith's files.</Note>
      <div>
        <button type="submit" disabled={busy || url.trim() === ""} style={primary}>
          Add
        </button>
      </div>
      {result && <Note tone={result.ok ? "good" : "error"}>{result.text}</Note>}
    </form>
  );
}

async function readBytes(file: File): Promise<Uint8Array> {
  if (typeof file.arrayBuffer === "function") return new Uint8Array(await file.arrayBuffer());
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(new Uint8Array(reader.result as ArrayBuffer));
    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}

function Uploads({ api, reload }: Props) {
  const [errors, setErrors] = useState<string[]>([]);
  const [note, setNote] = useState<{ ok: boolean; text: string } | null>(null);
  const [replacing, setReplacing] = useState<{ name: string; fileName: string; bytes: Uint8Array } | null>(null);
  const [copied, setCopied] = useState<string | null>(null);

  const send = async (fileName: string, bytes: Uint8Array, replace: boolean) => {
    setErrors([]);
    setNote(null);
    try {
      const status = await api.upload(fileName, bytes, replace);
      setNote({ ok: true, text: `Added: ${upcoming(status.eventCount)}.` });
      await reload();
    } catch (e) {
      const err = e as UploadError | string;
      if (typeof err === "string") setNote({ ok: false, text: err });
      else if (err.kind === "exists") setReplacing({ name: err.name, fileName, bytes });
      else if (err.kind === "invalid") setErrors(err.errors);
      else setNote({ ok: false, text: "The file is larger than 20 MB." });
    }
  };
  const take = async (file: File | undefined) => {
    if (!file) return;
    if (!/\.(json|ics)$/i.test(file.name)) {
      setErrors([]);
      setNote({ ok: false, text: "Use a .json schedule or an .ics calendar file." });
      return;
    }
    await send(file.name, await readBytes(file), false);
  };
  const copy = async (what: string, text: string) => {
    await copyText(text);
    setCopied(what);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      <div
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          e.preventDefault();
          void take(e.dataTransfer.files[0]);
        }}
        style={{ border: "1px dashed rgba(255,255,255,0.24)", borderRadius: 10, padding: 14, display: "flex", flexDirection: "column", gap: 8, fontSize: 14 }}
      >
        <span>Drop a timetable here: a Backlith .json schedule or an .ics calendar file.</span>
        <label style={{ display: "inline-flex", alignItems: "center", gap: 8 }}>
          <span style={{ ...secondary, display: "inline-flex", alignItems: "center" }}>Choose a file</span>
          <input type="file" aria-label="Choose a file" accept=".json,.ics" onChange={(e) => void take(e.target.files?.[0])} style={{ position: "absolute", width: 1, height: 1, opacity: 0 }} />
        </label>
      </div>
      <Note>To turn a timetable into a schedule file, copy the prompt, paste it into any LLM with your timetable, and drop the file it gives you here.</Note>
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <button type="button" style={secondary} onClick={() => void api.llmPrompt().then((p) => copy("prompt", p))}>
          Copy prompt for an LLM
        </button>
        {copied === "prompt" && <Note tone="good">Copied</Note>}
      </div>
      {note && <Note tone={note.ok ? "good" : "error"}>{note.text}</Note>}
      {errors.length > 0 && (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <Note tone="error">The file has these problems:</Note>
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13, color: "#FFC9BD", display: "flex", flexDirection: "column", gap: 4 }}>
            {errors.map((e, i) => (
              <li key={i}>{e}</li>
            ))}
          </ul>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <button type="button" style={secondary} onClick={() => void copy("errors", errors.join("\n"))}>
              Copy errors
            </button>
            {copied === "errors" && <Note tone="good">Copied</Note>}
          </div>
        </div>
      )}
      {replacing && (
        <div role="dialog" aria-modal="true" aria-label={`Replace “${replacing.name}”?`} style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.55)", display: "grid", placeItems: "center", zIndex: 10 }}>
          <div style={{ width: 380, maxWidth: "calc(100vw - 32px)", background: "#161B22", border: "1px solid rgba(255,255,255,0.12)", borderRadius: 12, padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
            <h2 style={{ margin: 0, fontSize: 17 }}>{`Replace “${replacing.name}”?`}</h2>
            <p style={{ margin: 0, fontSize: 14, color: "#C9D2DE" }}>A calendar with this name is already here. Replacing it keeps its colour and switch.</p>
            <div style={{ display: "flex", gap: 12, justifyContent: "flex-end" }}>
              <button type="button" style={secondary} onClick={() => setReplacing(null)}>
                Cancel
              </button>
              <button
                type="button"
                style={primary}
                onClick={() => {
                  const r = replacing;
                  setReplacing(null);
                  void send(r.fileName, r.bytes, true);
                }}
              >
                Replace
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export function SourcesPanel(props: Props) {
  const { state } = props;
  const statuses = new Map(state.sources.map((s) => [s.id, s]));
  return (
    <>
      <Section title="Calendars">
        {state.settings.calendars.length === 0 ? (
          <Note>No calendars yet. Add a calendar link or upload a timetable below.</Note>
        ) : (
          <ul style={{ margin: 0, padding: 0, display: "flex", flexDirection: "column", gap: 12 }}>
            {state.settings.calendars.map((c) => (
              <SourceRow key={c.id} source={c} status={statuses.get(c.id)} api={props.api} reload={props.reload} />
            ))}
          </ul>
        )}
      </Section>
      <Section title="Add a calendar link">
        <AddLinkForm {...props} />
      </Section>
      <Section title="Upload a timetable">
        <Uploads {...props} />
      </Section>
    </>
  );
}
