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
| `LocomotionTransition.IsCurrentFallSpeedTooFastToEnter` (245) | Falling-speed lock on double jump and dash, lifted by Legendary+ Reinforced Tendons. |
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
