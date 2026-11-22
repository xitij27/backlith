# Backlith

A Windows desktop background that holds useful tools. The first one shows the coming week from all of the author's calendars and how busy each day will be. See `README.md` for how it is meant to work.

## Read first

`docs/decisions.md` holds the research, what the author has said, the decisions and the open questions. Read it before working on the project.

## Keep all context here

Everything a later session would need goes into this repository and is committed with the work that produced it: what the author said and decided, what was found out, plans, and facts about the machines and tools. Not in an assistant's memory, a scratch folder or the conversation alone. Decisions and findings go in `docs/decisions.md` unless a better place is made for them.

## Licence

GPL-3.0-or-later. Start every new source file with the line `SPDX-License-Identifier: GPL-3.0-or-later`, written as a comment in that file's language. Code copied from elsewhere must be under a licence compatible with GPL-3.0, such as MIT, BSD, Apache-2.0 or GPL-3.0 itself.

## Secrets

Calendar links and login tokens never go into the repository or the wallpaper page. They live in a git-ignored local config file.
