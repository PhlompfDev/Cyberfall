# Tuning sources

Where the numbers in `tuning/pilot.toml` come from. Titanfall 2 values are in
engine units (0.0254 m).

## Titanfall 2 player settings

From `scripts/players/mp/*.set` in the game's VPKs (`pilot_mp.set` is
multiplayer, `pilot_solo.set` campaign). Multiplayer values are used.

| Setting | Multiplayer | Campaign | In `pilot.toml` |
|---|---|---|---|
| `stand.speed` | 173.5 | 162.5 | `walk_speed` |
| `stand.sprintspeed` | 260 | 243 | `sprint_speed` |
| `crouch.speed` | 80 | 80 | `crouch_speed` |
| `jumpheight` | 60 | 60 | `jump_speed` |
| `gravityscale` | 0.80 | 0.75 | `gravity` |
| `stepheight` | 18 | 18 | `step_height` |
| `wallrun_timeLimit` | 1.75 | 1.75 | `wallrun_time_limit` |
| `wallrunJumpOutwardSpeed` | 205 | 205 | `wall_kick_out_speed` |
| `wallrunJumpUpSpeed` | 230 | 230 | `wall_kick_up_speed` |
| `wallrunJumpInputDirSpeed` | 80 | 75 | `wall_kick_look_speed` |
| `wallrunMaxSpeedHorizontal` | 420 | 340 | (fitted instead) |
| `slidedecel` | 50 | 50 | `slide_decel` |
| `slidevelocitydecay` | 0.7 | 0.7 | `slide_decay_rate` |

Ground acceleration, friction, air control, slide boost and the wallrun curves
aren't in the settings files. They were fitted from recordings.

## Recordings

Recorded in a Northstar private match on `mp_forwardbase_kodai`, about 143
samples a second (see `telemetry/`).

| Behaviour | What the data shows |
|---|---|
| Ground | Linear acceleration (2500 units/s² net) and friction (1500 units/s²) |
| Gravity | 600 units/s² (750 with the 0.8 scale) |
| Jump | Launch speed about 300, rise 72 units |
| Air control | Speed along the wish direction topped up to about 60 at about 500 units/s² |
| Slide | `dv/dt = -(slidedecel + k v)`, `k = -ln(0.7)`; ends at about 126 |
| Wall-kick | Outward 205, up 230, plus 80 in the look direction along the wall |

## Wallrun fit

Fitted over the wall contacts in `wallrun`, `wall-kick` and `wall-chain`, each
scored in its own window (`telemetry/recordings/<take>@<window>.geometry.toml`).

| Behaviour | Values |
|---|---|
| Contact | Keep speed along the wall; vertical becomes `min(vz + 250, 250)` |
| Speed along the wall | Closes on 370 units/s at 3.6 /s |
| Vertical | Pulled at 5.5 /s toward `55 - 110 t` units/s |
| Steering away | Lets go past 0.74 (dot with the wall normal) |
| Falling off | Pushes about 70 units/s away from the wall |
