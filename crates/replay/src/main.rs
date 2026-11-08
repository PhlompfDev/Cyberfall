//! `replay report`: score every recording against the current tuning.
//! `replay compare <name> [every]`: recorded and simulated values side by side.

use std::fs;
use std::path::{Path, PathBuf};

use glam::Vec3;
use pilot_core::Tuning;
use replay::{geometry, score, simulate, Recording, SimTick, UNITS_TO_METRES};

const STEP: f32 = 1.0 / 60.0;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A recording plus an optional sidecar: `<take>.geometry.toml` or `<take>@<label>.geometry.toml`.
struct Case {
    name: String,
    recording: PathBuf,
    sidecar: Option<PathBuf>,
}

fn run(case: &Case, t: &Tuning) -> Result<Vec<SimTick>, String> {
    let path = &case.recording;
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut recording = Recording::parse(&text)?;
    let sidecar = case.sidecar.as_deref().map(geometry::load_sidecar).transpose()?;
    if let (Some(s), Some(first)) = (sidecar.as_ref(), recording.ticks.first().copied()) {
        let from = first.time + s.start.unwrap_or(0.0);
        let to = first.time + s.end.unwrap_or(f32::MAX);
        recording.ticks.retain(|tick| tick.time >= from && tick.time <= to);
    }
    let world = geometry::build(&recording, sidecar.as_ref(), t.collision_skin)?;
    Ok(simulate(&recording, &world, t, STEP))
}

fn cases(dir: &Path) -> Vec<Case> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    paths.sort();
    let mut out = Vec::new();
    for recording in paths.iter().filter(|p| {
        p.extension().is_some_and(|e| e == "txt") && p.file_stem().is_some_and(|s| s != "settings")
    }) {
        let stem = recording.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let windows: Vec<&PathBuf> = paths
            .iter()
            .filter(|p| {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                name.ends_with(".geometry.toml") && name.starts_with(&format!("{stem}@"))
            })
            .collect();
        if windows.is_empty() {
            let plain = recording.with_extension("geometry.toml");
            let sidecar = plain.exists().then_some(plain);
            out.push(Case { name: stem, recording: recording.clone(), sidecar });
        } else {
            for window in windows {
                let name = window.file_name().unwrap_or_default().to_string_lossy();
                let label = name.trim_end_matches(".geometry.toml").to_string();
                out.push(Case { name: label, recording: recording.clone(), sidecar: Some(window.clone()) });
            }
        }
    }
    out
}

/// Horizontal speed in engine units, as the game's own numbers are.
fn flat_speed(velocity: Vec3) -> f32 {
    Vec3::new(velocity.x, velocity.y, 0.0).length() / UNITS_TO_METRES
}

fn compare(case: &Case, every: usize, tuning: &Tuning) {
    let ticks = run(case, tuning).unwrap_or_else(|e| panic!("{e}"));
    let t0 = ticks.first().map(|t| t.time).unwrap_or(0.0);
    println!("  time | recorded speed      vz mode     | simulated speed      vz mode      | apart (units)");
    for tick in ticks.iter().step_by(every.max(1)) {
        let (r, s) = (tick.recorded, tick.state);
        println!(
            "{:6.2} | {:14.1} {:7.1} {:8} | {:15.1} {:7.1} {:11} | {:7.1}",
            tick.time - t0,
            flat_speed(r.velocity),
            r.velocity.z / UNITS_TO_METRES,
            format!("{:?}", r.mode),
            flat_speed(s.velocity),
            s.velocity.z / UNITS_TO_METRES,
            format!("{:?}", s.mode),
            (s.position - r.position).length() / UNITS_TO_METRES,
        );
    }
}

fn report(dir: &Path, tuning: &Tuning) {
    println!("| Recording | Speed RMS | Position | Mode mismatches | Result |");
    println!("|---|---|---|---|---|");
    for case in cases(dir) {
        let name = &case.name;
        match run(&case, tuning) {
            Ok(ticks) => {
                let s = score(&ticks);
                println!(
                    "| {name} | {:.1}% | {:.1}% | {} | {} |",
                    s.speed_rms * 100.0,
                    s.position_error * 100.0,
                    s.mode_mismatches.len(),
                    if s.passed() { "PASS" } else { "FAIL" }
                );
            }
            Err(error) => println!("| {name} | | | | ERROR: {error} |"),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tuning =
        pilot_tuning::load(&repo().join("tuning/pilot.toml")).expect("tuning/pilot.toml must load");
    let dir = repo().join("telemetry/recordings");
    match args.first().map(String::as_str) {
        Some("compare") => {
            let name = args.get(1).expect("usage: replay compare <name> [every]");
            let every = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(6);
            let case = cases(&dir).into_iter().find(|c| &c.name == name).expect("no such recording or window");
            compare(&case, every, &tuning);
        }
        _ => report(&dir, &tuning),
    }
}
