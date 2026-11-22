# Backlith

The monolith behind your windows.

A desktop background that does useful work. It holds tools, laid out by the app as one background, not as a set of widgets, so nothing has to be placed by hand. The first tool shows the coming week from all of your calendars, and how busy each day is going to be.

Windows comes first. Ubuntu may follow.

## Status

Nothing is built yet. The repository holds the research and the decisions so far, in [docs/decisions.md](docs/decisions.md). The language for the script is not chosen yet.

## How it is meant to work

1. A script runs every 10 minutes from Windows Task Scheduler. It fetches the calendars, works out the week, writes the data next to the wallpaper page, and exits. No server keeps running.
2. [Lively Wallpaper](https://github.com/rocksdanister/lively) shows the page as the live wallpaper, behind the desktop icons. Lively is a tray app that starts with Windows. There is no browser tab to close.
3. On each run the script also draws the page to a PNG and sets it as the normal Windows wallpaper. If Lively is not running, the desktop still shows the week, at most 10 minutes old.

## Secrets

Calendar links (secret ICS addresses) and login tokens give full read access to a calendar. They live in a local config file that git ignores, never in the repository or in the wallpaper page.

## License

Copyright (C) 2026 Kshitij Parashar

Backlith is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. It is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See [LICENSE](LICENSE) for the full text.
