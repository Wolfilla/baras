# v2026.2.9000

## Timers

**Creating timers is a lot of work and it's easy to make mistakes, if you see something off please
report it so we can update the defaults**

- Apex Vanguard Mass Target Lock P4 timer no longer fires on combat start
- XR-53 timers have been overhauled by Keetsune. Several are disabled by default on his recommendation.
- Timers for all bosses in Gods has been updated. Thank you to Error for working on this.

## Encounter Classification

- Non-battle revives of the local player will now automatically end the combat encounter
- Encounters ended by exiting the Area are now classified as wipes unless there is a known special condition (e.g. TFB excluded)
- Healing actions are ignored when evaluating fallback encounter timeout

## Data Explorer

- Increased font size of ability usage tab
- Combat log IDs now show before the associated name

## Performance

- Cache data-explorer icons to reduce memory pressure from frequent lookups
