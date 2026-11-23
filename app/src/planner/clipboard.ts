// SPDX-License-Identifier: GPL-3.0-or-later
// WebView2 and WebKitGTK both support the async clipboard API.
export async function copyText(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
}
