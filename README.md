# Cyberfall

Titanfall 2 wallrunning for Cyberpunk 2077.

## Layout

| Path | What |
|---|---|
| `mod/` | The game files: scripts (`r6/scripts`), tweaks (`r6/tweaks`) and the settings menu (CET, `bin/x64/plugins/cyber_engine_tweaks/mods/Cyberfall`) |
| `crates/cyberfall-plugin` | RED4ext plugin the scripts call into |
| `crates/cyberfall-bridge` | Cyberpunk-side wallrun logic, testable without the game |
| `crates/pilot-core` | Titanfall 2 movement simulation, including the wall controller |
| `tuning/pilot.toml` | Every movement number, compiled into the plugin |
| `crates/replay`, `telemetry/` | Real Titanfall 2 recordings and the harness that scores pilot-core against them |
| `crates/gym` | A test course to play pilot-core on its own |
| `docs/` | Game hooks, tuning sources, pilot-core host interface |

## Building

Needs Rust, CMake and LLVM (for red4ext-rs).

```
cargo test
scripts/install-dev.ps1     # build the plugin and install everything into the game
scripts/check-scripts.ps1   # compile the scripts with the game's compiler
```

Requires RED4ext, redscript, TweakXL and Codeware in game. The settings menu needs Cyber Engine Tweaks and Native Settings UI.
