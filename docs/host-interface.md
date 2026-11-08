# Hosting pilot-core

A host is whatever runs the full `pilot-core` simulation, such as the gym. The
Cyberpunk mod only uses the wall controller (`wall::wall_step`) through
`cyberfall-bridge`.

## Each fixed tick

1. Build an `Input` from the player's controls.
2. Call `step(&state, &input, &tuning, &world, dt)`.
3. Keep the returned `PilotState`; react to the returned `Event`s.
4. Place the player at `state.position` and the camera at
   `state.position + up * state.camera.eye_height`, rolled by
   `state.camera.roll`.

`step` must be called at a fixed `dt` (1/60 s by default). A host that renders
faster interpolates between the last two states; it never calls `step` with a
variable `dt`.

## Conventions

- Metres, seconds, radians. Z is up, right-handed.
- Yaw 0 faces +X; yaw increases turning left; at yaw 0 right is -Y.
- `position` is the feet: the bottom of the capsule.
- `camera.roll` positive tilts the view clockwise (right ear down).
- `Button.pressed` is true only on the tick the button went down. A host that
  polls input more often than it ticks must latch presses until the next tick.

## The World trait

Positions passed to `sweep` and `ground` are capsule centres. The capsule is
upright; `half_height` is half its cylinder section.

| Method | Must return |
|---|---|
| `sweep(capsule, from, to)` | The first contact moving the capsule from `from` to `to`: `fraction` in 0..1 along the path, the contact `point`, and the unit `normal` of the surface, pointing away from it. `None` if the path is clear. A capsule already overlapping something reports a hit at fraction 0. |
| `raycast(from, to)` | The first surface a ray meets, in the same form. A ray starting inside solid geometry reports a hit at fraction 0. |
| `ground(capsule, centre, max_drop)` | Optional override. The default sweeps the capsule straight down. |

Normals must be the true surface normals. `pilot-core` uses them to decide
what is floor (`max_slope_deg`) and what is wall (`wall_max_normal_z`).

## What pilot-core does itself

Collision response (sliding along surfaces, climbing steps, sticking to
slopes), every movement mode and every transition between them. A host does
not move the player, apply gravity or resolve collisions.

Pilots take no fall damage. `Landed` reports the impact speed for sound and
camera effects only; a host must not turn it into damage.

## Events

`Jumped`, `DoubleJumped`, `Landed { impact_speed }`, `SlideStarted`,
`SlideEnded`, `WallrunStarted`, `WallrunEnded`, `WallKicked`, `MantleStarted`,
`MantleEnded`. They are for sound, animation and effects; ignoring them does
not change the simulation.
