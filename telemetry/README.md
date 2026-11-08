# Recording Titanfall 2 movement

The `Cyberfall.Telemetry` Northstar mod records the pilot's position, velocity,
view and inputs every rendered frame. The recordings in `recordings/` are what
the movement is tuned and tested against (`cargo run -p replay -- report`).

## Install

1. Install Northstar.
2. Copy `northstar-mod/Cyberfall.Telemetry` into `Titanfall2/R2Northstar/mods/`.

## Record

1. Start a private match on a map with flat ground and parallel walls.
2. In the console, run `cf_rec settings` once to save the live movement settings.
3. For each take: `cf_rec start <name>`, do the move, `cf_rec stop`.

Files are saved as `.txt` in `Titanfall2/R2Northstar/save_data/Cyberfall.Telemetry/`.
Copy them into `recordings/`. Crouch is read from Left Ctrl, so use hold crouch.

## Takes

| Name | Move |
|---|---|
| `sprint` | Sprint 3 seconds from a standstill, then release and coast to a stop |
| `walk` | The same without sprint |
| `strafe-stop` | Sprint, then release forward for 2 seconds |
| `jump` | Jump and land, then jump and double jump at the peak |
| `sprint-jump` | Sprint, jump, land, keep sprinting |
| `air-strafe` | Sprint, jump, release forward, hold right and turn right 90 degrees |
| `slide` | Sprint, then hold crouch until the slide ends |
| `slide-hop` | Sprint, slide, jump out of the slide, slide on landing; five hops |
| `wallrun` | Sprint, jump into a wall at a shallow angle, hold forward until you fall off |
| `wall-kick` | As `wallrun`, but jump off after about a second |
| `wall-chain` | Four alternating wall-kicks between two parallel walls |
| `mantle` | Run at a ledge about head height |
