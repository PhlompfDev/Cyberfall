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
