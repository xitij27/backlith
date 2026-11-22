# Calendar wallpaper

A desktop background that shows the coming week from all of your calendars, and how busy each day is going to be. It is the background itself, not a set of widgets, so the app lays everything out and nothing has to be placed by hand.

Windows comes first. Ubuntu may follow.

## Status

Nothing is built yet. The repository holds the research and the decisions so far, in [docs/decisions.md](docs/decisions.md). The language for the script is not chosen yet.

## How it is meant to work

1. A script runs every 10 minutes from Windows Task Scheduler. It fetches the calendars, works out the week, writes the data next to the wallpaper page, and exits. No server keeps running.
2. [Lively Wallpaper](https://github.com/rocksdanister/lively) shows the page as the live wallpaper, behind the desktop icons. Lively is a tray app that starts with Windows. There is no browser tab to close.
3. On each run the script also draws the page to a PNG and sets it as the normal Windows wallpaper. If Lively is not running, the desktop still shows the week, at most 10 minutes old.

## Secrets

Calendar links (secret ICS addresses) and login tokens give full read access to a calendar. They live in a local config file that git ignores, never in the repository or in the wallpaper page.
