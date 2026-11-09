// Provided by red4ext/plugins/Cyberfall/cyberfall.dll.

public native func CyberfallWallVersion() -> String;
public native func CyberfallWallSetSettings(enabled: Bool, needsPerk: Bool, length: Float, speed: Float, kick: Float, tilt: Float) -> Void;
public native func CyberfallWallEnabled() -> Bool;
public native func CyberfallWallNeedsPerk() -> Bool;
public native func CyberfallWallSetVelocity(x: Float, y: Float, z: Float) -> Void;
public native func CyberfallWallSetReading(hasWall: Bool, nx: Float, ny: Float, nz: Float, distance: Float, heights: Int32) -> Void;
public native func CyberfallWallSetInput(forward: Float, right: Float, facingX: Float, facingY: Float, jump: Bool, crouch: Bool) -> Void;
public native func CyberfallWallStep(groundClear: Bool, dt: Float) -> Bool;
public native func CyberfallWallIsRunning() -> Bool;
public native func CyberfallWallStarted() -> Bool;
// 0 none, 1 kicked, 2 timed out, 3 let go, 4 lost the wall
public native func CyberfallWallEnded() -> Int32;
public native func CyberfallWallRefreshAirMoves() -> Bool;
public native func CyberfallWallKickWindow() -> Bool;
public native func CyberfallWallVelocityX() -> Float;
public native func CyberfallWallVelocityY() -> Float;
public native func CyberfallWallVelocityZ() -> Float;
public native func CyberfallWallRoll() -> Float;
public native func CyberfallWallAdvanceAir(dt: Float) -> Void;
public native func CyberfallWallTouchedGround() -> Void;
public native func CyberfallWallReset() -> Void;
public native func CyberfallWallAirDash() -> Void;
public native func CyberfallWallSetAhead(blocked: Bool, distance: Float, clearance: Float) -> Void;
public native func CyberfallWallMaxNormalZ() -> Float;
public native func CyberfallWallBumpLookahead() -> Float;
public native func CyberfallWallBumpMaxClearance() -> Float;
// Radians
public native func CyberfallWallCameraRoll(dt: Float) -> Float;
public native func CyberfallWallGroundClearance() -> Float;
public native func CyberfallWallStickDistance() -> Float;
// Charge-jump legs: the wall kick waits for Jump to be let go and grows with the hold.
public native func CyberfallWallSetCharge(hasChargeJump: Bool, jumpHeld: Bool) -> Void;
public native func CyberfallWallCharging() -> Bool;
// Charge tier of the most recent kick: 0 normal, 3 full.
public native func CyberfallWallChargeTier() -> Int32;

// Momentum slide.
public native func CyberfallSlideSetSettings(enabled: Bool, speed: Float) -> Void;
public native func CyberfallSlideEnabled() -> Bool;
public native func CyberfallSlideNoteAir(vx: Float, vy: Float, crouchPressed: Bool, crouchHeld: Bool, toggleTap: Bool, dt: Float) -> Void;
public native func CyberfallSlideWantsLanding() -> Bool;
// now: game time, seconds.
public native func CyberfallSlideBegin(vx: Float, vy: Float, now: Float, fromLanding: Bool) -> Void;
public native func CyberfallSlideSetInput(forward: Float, right: Float, facingX: Float, facingY: Float, crouchHeld: Bool) -> Void;
// Ground normal zero when none was seen. Returns whether the slide goes on.
public native func CyberfallSlideStep(vx: Float, vy: Float, vz: Float, nx: Float, ny: Float, nz: Float, dt: Float) -> Bool;
public native func CyberfallSlideVelocityX() -> Float;
public native func CyberfallSlideVelocityY() -> Float;
public native func CyberfallSlideVelocityZ() -> Float;
public native func CyberfallSlideActive() -> Bool;
public native func CyberfallSlideEnd() -> Void;
public native func CyberfallSlideLanded() -> Void;
// While sprinting: V's sprint speed, the slowest a fast slide eases down to.
public native func CyberfallSlideSetSprintSpeed(speed: Float) -> Void;
