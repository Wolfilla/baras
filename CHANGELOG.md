# v2026.2.1900

Documentation is in the process of bring improved at baras-app.github.io. Some tutorials added.

## Auto-hide When not live

Added an experimental option to hide overlays when the application is not live or the game is not running.
This will monitor for swtor.exe running on your system. If the feature is disabled no process monitoring occurs.

## UI

- Added option for European-style `1.234,567` number formatting in the general application settings
- Scroll bars on Windows now fit better with the application UI
- Use toggle buttons instead of checkboxes
- Fixed issue where UI was not responsive to saved changes in encounter editor non-timer elements

## Effects Tracker

- Effects audio will not play if disabled
- Ability tracking logic improved to be more robust
- Fixed issue causing local player discipline to not be recognized for effects scoping
- Effects can now be configured to instantly alert in the same manner as encounter timers

## Overlays

- Improved text alignment and separator spacing of personal overlay
- Fixed issue with text overflow in DOT tracker when using font scaling
- Fixed issue with class/role icons not displaying on certain frame spacing settings

## Encounter Definitions

- Removed spammy Watchdog Missiles timers
- Added Xenoanalyst to boss definitions
- Added phases to Cartel Warlords
- Added alert when targeted by Huntmaster grenade
- Added initial phases and timers for Izax
- Reformatted Apex Vanguard phases - timer update planned soon.
- Added Brontes arcing assault timer

Thanks Wolfy and Errør for contribution to the encounter definitions!
