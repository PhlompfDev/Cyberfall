use std::fs;
use std::path::{Path, PathBuf};

const ALLOWED_DEPS: [&str; 2] = ["glam", "serde"];
const FORBIDDEN_IN_SRC: [&str; 9] = [
    "bevy", "parry", "red4ext", "winit", "std::fs", "std::time", "std::net", "std::env",
    "std::thread",
];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn dependency_names() -> Vec<String> {
    let manifest = fs::read_to_string(crate_dir().join("Cargo.toml")).unwrap();
    let mut names = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
        } else if in_deps && !line.is_empty() && !line.starts_with('#') {
            let name = line.split(['.', '=', ' ']).next().unwrap();
            names.push(name.to_string());
        }
    }
    names
}

#[test]
fn guard_targets_exist() {
    let mut files = Vec::new();
    rust_files(&crate_dir().join("src"), &mut files);
    assert!(files.len() >= 2, "src/ should hold the crate's modules");
    assert!(!dependency_names().is_empty(), "[dependencies] section not found");
}

#[test]
fn only_glam_and_serde_are_dependencies() {
    for name in dependency_names() {
        assert!(ALLOWED_DEPS.contains(&name.as_str()), "forbidden dependency: {name}");
    }
}

#[test]
fn source_never_touches_a_host_or_the_os() {
    let mut files = Vec::new();
    rust_files(&crate_dir().join("src"), &mut files);
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        for word in FORBIDDEN_IN_SRC {
            assert!(!text.contains(word), "{} mentions `{word}`", file.display());
        }
    }
}
