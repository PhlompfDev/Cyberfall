//! RED4ext plugin: the wall controller as global natives for the scripts.
//! Logic lives in `cyberfall-bridge`.

use std::sync::{Mutex, MutexGuard, OnceLock};

use cyberfall_bridge::Bridge;
use red4ext_rs::{
    Exportable, GlobalExport, Plugin, SemVer, U16CStr, export_plugin_symbols, exports, global,
    wcstr,
};

pub struct Cyberfall;

impl Plugin for Cyberfall {
    const AUTHOR: &'static U16CStr = wcstr!("Cyberfall");
    const NAME: &'static U16CStr = wcstr!("Cyberfall");
    const VERSION: SemVer = SemVer::new(1, 1, 0);

    fn exports() -> impl Exportable {
        exports![
            GlobalExport(global!(c"CyberfallWallVersion", version)),
            GlobalExport(global!(c"CyberfallWallSetSettings", set_settings)),
            GlobalExport(global!(c"CyberfallWallEnabled", enabled)),
            GlobalExport(global!(c"CyberfallWallNeedsPerk", needs_perk)),
            GlobalExport(global!(c"CyberfallWallSetVelocity", set_velocity)),
            GlobalExport(global!(c"CyberfallWallSetReading", set_reading)),
            GlobalExport(global!(c"CyberfallWallSetInput", set_input)),
            GlobalExport(global!(c"CyberfallWallStep", step)),
            GlobalExport(global!(c"CyberfallWallIsRunning", running)),
            GlobalExport(global!(c"CyberfallWallStarted", started)),
            GlobalExport(global!(c"CyberfallWallEnded", ended)),
            GlobalExport(global!(c"CyberfallWallRefreshAirMoves", refresh_air_moves)),
            GlobalExport(global!(c"CyberfallWallKickWindow", kick_window)),
            GlobalExport(global!(c"CyberfallWallVelocityX", velocity_x)),
            GlobalExport(global!(c"CyberfallWallVelocityY", velocity_y)),
            GlobalExport(global!(c"CyberfallWallVelocityZ", velocity_z)),
            GlobalExport(global!(c"CyberfallWallRoll", roll)),
            GlobalExport(global!(c"CyberfallWallAdvanceAir", advance_air)),
            GlobalExport(global!(c"CyberfallWallTouchedGround", touched_ground)),
            GlobalExport(global!(c"CyberfallWallReset", reset)),
            GlobalExport(global!(c"CyberfallWallAirDash", air_dash)),
            GlobalExport(global!(c"CyberfallWallSetAhead", set_ahead)),
            GlobalExport(global!(c"CyberfallWallMaxNormalZ", max_normal_z)),
            GlobalExport(global!(c"CyberfallWallBumpLookahead", bump_lookahead)),
            GlobalExport(global!(c"CyberfallWallBumpMaxClearance", bump_max_clearance)),
            GlobalExport(global!(c"CyberfallWallCameraRoll", camera_roll)),
            GlobalExport(global!(c"CyberfallWallGroundClearance", ground_clearance)),
            GlobalExport(global!(c"CyberfallWallStickDistance", stick_distance))
        ]
    }
}

export_plugin_symbols!(Cyberfall);

fn bridge() -> MutexGuard<'static, Bridge> {
    static BRIDGE: OnceLock<Mutex<Bridge>> = OnceLock::new();
    BRIDGE
        .get_or_init(|| Mutex::new(Bridge::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn set_settings(enabled: bool, needs_perk: bool, length: f32, speed: f32, kick: f32, tilt: f32) {
    bridge().set_settings(enabled, needs_perk, length, speed, kick, tilt);
}

fn enabled() -> bool {
    bridge().enabled()
}

fn needs_perk() -> bool {
    bridge().needs_perk()
}

fn set_velocity(x: f32, y: f32, z: f32) {
    bridge().set_velocity(x, y, z);
}

fn set_reading(has_wall: bool, nx: f32, ny: f32, nz: f32, distance: f32, heights: i32) {
    bridge().set_reading(has_wall, nx, ny, nz, distance, heights);
}

fn set_input(forward: f32, right: f32, facing_x: f32, facing_y: f32, jump: bool, crouch: bool) {
    bridge().set_input(forward, right, facing_x, facing_y, jump, crouch);
}

fn step(ground_clear: bool, dt: f32) -> bool {
    bridge().step(ground_clear, dt)
}

fn running() -> bool {
    bridge().running()
}

fn started() -> bool {
    bridge().started()
}

fn ended() -> i32 {
    bridge().ended()
}

fn refresh_air_moves() -> bool {
    bridge().refresh_air_moves()
}

fn kick_window() -> bool {
    bridge().kick_window_open()
}

fn velocity_x() -> f32 {
    bridge().velocity().x
}

fn velocity_y() -> f32 {
    bridge().velocity().y
}

fn velocity_z() -> f32 {
    bridge().velocity().z
}

fn roll() -> f32 {
    bridge().roll()
}

fn advance_air(dt: f32) {
    bridge().advance_air(dt);
}

fn touched_ground() {
    bridge().touched_ground();
}

fn reset() {
    bridge().reset();
}

fn air_dash() {
    bridge().air_dash();
}

fn set_ahead(blocked: bool, distance: f32, clearance: f32) {
    bridge().set_ahead(blocked, distance, clearance);
}

fn max_normal_z() -> f32 {
    bridge().wall_max_normal_z()
}

fn bump_lookahead() -> f32 {
    bridge().bump_lookahead()
}

fn bump_max_clearance() -> f32 {
    bridge().bump_max_clearance()
}

fn camera_roll(dt: f32) -> f32 {
    bridge().camera_roll(dt)
}

fn ground_clearance() -> f32 {
    bridge().ground_clearance()
}

fn stick_distance() -> f32 {
    bridge().stick_distance()
}
