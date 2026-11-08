//! Wall recordings the fitted model must keep passing. The wall-chain windows
//! are only scored by `replay report`.

use std::fs;
use std::path::PathBuf;

use replay::{geometry, score, simulate, Recording};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn score_window(take: &str, window: &str) -> replay::Score {
    let dir = repo().join("telemetry/recordings");
    let t = pilot_tuning::load(&repo().join("tuning/pilot.toml")).unwrap();
    let text = fs::read_to_string(dir.join(format!("{take}.txt"))).unwrap();
    let mut recording = Recording::parse(&text).unwrap();
    let sidecar = geometry::load_sidecar(&dir.join(format!("{take}@{window}.geometry.toml"))).unwrap();
    let first = recording.ticks[0].time;
    let (from, to) = (first + sidecar.start.unwrap_or(0.0), first + sidecar.end.unwrap_or(f32::MAX));
    recording.ticks.retain(|tick| tick.time >= from && tick.time <= to);
    let world = geometry::build(&recording, Some(&sidecar), t.collision_skin).unwrap();
    score(&simulate(&recording, &world, &t, 1.0 / 60.0))
}

#[test]
fn the_long_wallrun_matches_titanfall() {
    let s = score_window("wallrun", "run");
    assert!(s.passed(), "{s:?}");
}

#[test]
fn the_wallrun_and_kick_match_titanfall() {
    let s = score_window("wall-kick", "run");
    assert!(s.passed(), "{s:?}");
}
