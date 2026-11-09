# Game hooks

What the scripts in `mod/r6/scripts/Cyberfall` hook, for Cyberpunk 2077 2.5.
Line numbers are in the decompiled scripts (`codeberg.org/adamsmasher/cyberpunk`),
`cyberpunk/player/psm/locomotionTransitions.swift` unless noted.

## Moving V

Each air frame the mod reads V's velocity (`DefaultTransition.GetLinearVelocity`),
asks the plugin for the wallrun velocity, and hands the difference back through
`stateContext.SetTemporaryVectorParameter(n"impulse", delta, true)`. While running,
gravity is zeroed with a temporary `locomotionParameters` and restored with
`SetLocomotionParameters`. The game's own controller does the collision.

## Hooks

| Hook | Why |
|---|---|
| `LocomotionAirEvents.OnUpdate` (3672) | The per-frame wallrun update. Every air state inherits or calls it. |
| `LocomotionAirEvents.OnExit` | Restore gravity. |
| `LocomotionGroundEvents.OnEnter` (664) | Landing frees the same-wall rule. The air dash (`dodge`, `DodgeEvents` 2279) is a ground state class, so this also fires mid-air. |
| `DoubleJumpDecisions.EnterCondition` (4244) | Jump on a wall is a kick, not a double jump. |
| `FallDecisions.EnterCondition` (3828) | `doubleJump` has no transition to itself, so V drops to `fall` once a wall gives the double jump back. |
| `LocomotionTransition.IsCurrentFallSpeedTooFastToEnter` (245) | Falling-speed lock on double jump and dash, lifted by Legendary+ Reinforced Tendons and Fortified Ankles. |
| `ChargeJumpDecisions.EnterCondition` (4310) | No base-game charge jump while charging a wall kick. |
| `SlideDecisions.EnterCondition` (1920), `StandDecisions.ToSlide` (917) | An armed landing goes straight into a slide. |
| `CrouchDecisions.OnAttach` (1083) | Keeps the slide decisions, to switch their enter condition on for a landing slide. |
| `SlideDecisions.ShouldExit` | Leave the slide when the plugin ends it. |
| `SlideEvents.OnEnter` / `OnUpdate` / `OnExit` (2045) | The per-frame slide update, also used by `slideAfterFall`. |
| `LocomotionEventsTransition.OnUpdate` (429) | Sprint speed for the slide, and forgetting a fall that didn't slide. |
| `PerkDisplayTooltipController.GetLevelDescription` (`UI/fullscreen/perks/perksMenuTooltips.swift` 250) | Air Dash perk text. |

## Game state the mod reads or writes

- Air Dash perk: `PlayerDevelopmentSystem.IsNewPerkBought(owner, Reflexes_Central_Milestone_3) == 3` (278).
- Air move refresh: permanent ints `currentNumberOfJumps`, `currentNumberOfAirDodges` and the permanent bool `disableAirDash` (680).
- Landing type: permanent int `LandingType` (`LandingType` enum: 2 hard, 3 very hard), set each air frame (3760).
- Walls: `SpatialQueriesSystem.SyncRaycastByCollisionGroup(..., n"Static", ...)`, static geometry only. Horizontal rays can return up- or down-facing normals, so hits are filtered by normal.
- Camera lean: `GetFPPCameraComponent().SetLocalOrientation`.
- Inputs: `IsActionJustPressed(n"Jump")`, `GetActionValue(n"MoveX")` / `n"MoveY"`.
- Legs cyberware: `EquipmentSystem.GetData(player).GetActiveItem(gamedataEquipmentArea.LegsCW)`.

Cyberpunk is Z-up in metres like pilot-core, but its forward is +Y, so the facing
is passed as a vector and the plugin takes `atan2(y, x)`.

## Slides, landings and charge jumps

Measured with SmoothMovement, SmootherDodgeAndDash and Immersive First Person installed.

- **A landing is `fall`/`jump` → `regularLand` for one frame → `stand` or `sprint`.**
  Horizontal speed barely changes through `regularLand`; the following `stand`/`sprint`
  bleeds it back down to sprint speed (8.25 m/s here) over about 0.3 s. No base-game
  path goes from a landing into `slide`.
- **Into a slide:** from `sprint` the game asks `SlideDecisions.EnterCondition`; from
  `stand` it asks `StandDecisions.ToSlide` (true only with the `JustDodgedLocomotion` tag,
  just after a ground dodge).
- **`slideFall` and `slideAfterFall`** are a slide that leaves a slope or edge and lands
  again. Neither has its own script class: both run `SlideEvents`/`SlideDecisions`.
- **An impulse works in the slide state.** A `+2 m/s` push set every frame through the
  `impulse` temporary vector parameter added about 2 m/s per frame.
- **Horizontal speed is capped at 15 m/s on the ground and 18 m/s in the air** in this
  setup; slides, landings and jumps all flatten at exactly those values.
- **Jump leaves a slide on the press frame** (no `minTimeToExit` wait), at the slide's speed.
- **Toggle crouch:** `GetActionValue(n"Crouch")` stays 0; a tap is
  `IsActionJustTapped(n"ToggleCrouch")` and sets the condition bool `CrouchToggled` on the
  ground. In the air the tap registers but `CrouchToggled` stays false, and a jump clears it.
- **A temporary `impulse` is applied on two frames if it isn't set again.** On the frame a
  wall kick ends a run the mod sets one impulse and stops; the next frame's velocity shows
  the same push a second time (a kick's up speed lands as about 2×). Frames that set an
  impulse every frame (the run, the slide) overwrite it and see it once.
- **The base game's charge jump can start from the `jump` state in the air** when Jump is
  released after a long hold. Releasing Jump while wallrunning never started one.
- **Fortified Ankles** is `Items.AdvancedReinforcedMuscles<Quality>`, `cyberwareType`
  `ReinforcedMuscles`, and gives `HasChargeJump`.
