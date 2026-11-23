# Data files

## windows-zones.csv

Maps Windows time zone names, which Outlook and Exchange write into calendar
files (for example `TZID:W. Europe Standard Time`), to IANA zone names.

Made on 23 November 2026 from the `territory="001"` rows of CLDR's
`common/supplemental/windowsZones.xml`
(https://github.com/unicode-org/cldr), taking the first IANA name of each.

CLDR is © Unicode, Inc., under the Unicode License v3
(https://www.unicode.org/license.txt), which is compatible with the GPL.
