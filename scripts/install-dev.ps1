# Build the plugin and install it with the scripts, tweaks and settings menu.
# Needs CMake and LLVM. Set CYBERPUNK_DIR to override the game folder.
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$game = if ($env:CYBERPUNK_DIR) { $env:CYBERPUNK_DIR } else { 'C:\Program Files (x86)\Steam\steamapps\common\Cyberpunk 2077' }
$env:PATH = "C:\Program Files\CMake\bin;$env:PATH"
if (-not $env:LIBCLANG_PATH) { $env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin' }

# Cargo reports progress on stderr, which Windows PowerShell treats as an error.
$ErrorActionPreference = 'Continue'
cargo build --release --manifest-path "$repo\crates\cyberfall-plugin\Cargo.toml" 2>&1 | Out-Host
$ErrorActionPreference = 'Stop'
if ($LASTEXITCODE -ne 0) { throw "plugin build failed" }

$plugins = Join-Path $game 'red4ext\plugins\Cyberfall'
$scripts = Join-Path $game 'r6\scripts\Cyberfall'
$tweaks = Join-Path $game 'r6\tweaks\Cyberfall'
$cet = Join-Path $game 'bin\x64\plugins\cyber_engine_tweaks\mods\Cyberfall'
New-Item -ItemType Directory -Force $plugins, $scripts, $tweaks, $cet | Out-Null
Copy-Item "$repo\crates\cyberfall-plugin\target\release\cyberfall_plugin.dll" (Join-Path $plugins 'cyberfall.dll') -Force
Copy-Item "$repo\mod\r6\scripts\Cyberfall\*.reds" $scripts -Force
Copy-Item "$repo\mod\r6\tweaks\Cyberfall\*.yaml" $tweaks -Force
Copy-Item "$repo\mod\bin\x64\plugins\cyber_engine_tweaks\mods\Cyberfall\init.lua" $cet -Force
Write-Host "Installed Cyberfall into $game"
