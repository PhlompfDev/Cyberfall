use std::fs;
use std::path::PathBuf;

const SOURCES: [&str; 5] = ["telemetry", "tf2-settings", "published", "host-scale", "estimate"];

fn toml_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tuning/pilot.toml")
}

fn is_value_line(line: &str) -> bool {
    let Some((key, _)) = line.split_once(" = ") else { return false };
    !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[test]
fn the_repo_file_loads() {
    let tuning = pilot_tuning::load(&toml_path()).unwrap();
    assert!(tuning.gravity > 0.0);
    assert!(tuning.sprint_speed > tuning.walk_speed);
    assert!(tuning.walk_speed > tuning.crouch_speed);
    assert!(tuning.stand_height > tuning.crouch_height);
    assert!(tuning.slide_entry_speed > tuning.slide_exit_speed);
    assert!(tuning.mantle_max_height > tuning.mantle_min_height);
}

#[test]
fn every_value_names_an_allowed_source() {
    let text = fs::read_to_string(toml_path()).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let mut values = 0;
    for (i, line) in lines.iter().enumerate() {
        if !is_value_line(line) {
            continue;
        }
        values += 1;
        let above = lines[..i].iter().rev().find(|l| !l.trim().is_empty()).copied().unwrap_or("");
        let source = above.strip_prefix("# source: ").unwrap_or_else(|| {
            panic!("line {}: `{line}` has no `# source:` line directly above it", i + 1)
        });
        let label = source.split_whitespace().next().unwrap_or("");
        assert!(SOURCES.contains(&label), "line {}: unknown source `{label}`", i + 1);
    }
    assert!(values >= 40, "expected the full tuning set, found {values} values");
}

#[test]
fn unknown_and_missing_keys_are_rejected() {
    let text = fs::read_to_string(toml_path()).unwrap();
    assert!(pilot_tuning::parse(&text).is_ok());

    let with_extra = format!("{text}\nnot_a_real_key = 1.0\n");
    assert!(pilot_tuning::parse(&with_extra).is_err());

    let without_gravity: String =
        text.lines().filter(|l| !l.starts_with("gravity = ")).collect::<Vec<_>>().join("\n");
    assert_ne!(without_gravity.len(), text.len(), "the gravity line was not found");
    assert!(pilot_tuning::parse(&without_gravity).is_err());
}

#[test]
fn spawn_starts_standing_on_the_ground_mode() {
    let tuning = pilot_tuning::load(&toml_path()).unwrap();
    let state = pilot_core::PilotState::spawn(glam::Vec3::new(1.0, 2.0, 3.0), &tuning);
    assert_eq!(state.position, glam::Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(state.mode, pilot_core::Mode::Grounded);
    assert_eq!(state.camera.eye_height, tuning.stand_eye_height);
    assert!(!state.crouched);
    assert_eq!(state.jumps_used, 0);
}

#[test]
fn capsules_match_the_configured_heights() {
    let tuning = pilot_tuning::load(&toml_path()).unwrap();
    let stand = tuning.stand_capsule();
    let crouch = tuning.crouch_capsule();
    assert!((2.0 * stand.centre_offset() - tuning.stand_height).abs() < 1e-6);
    assert!((2.0 * crouch.centre_offset() - tuning.crouch_height).abs() < 1e-6);
    assert_eq!(stand.radius, tuning.capsule_radius);
}
