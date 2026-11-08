# Compile the mod's scripts with the game's compiler, in a temp folder.
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$game = if ($env:CYBERPUNK_DIR) { $env:CYBERPUNK_DIR } else { 'C:\Program Files (x86)\Steam\steamapps\common\Cyberpunk 2077' }
$work = Join-Path ([IO.Path]::GetTempPath()) "cyberfall-scc-$PID"
New-Item -ItemType Directory -Force "$work\scripts\Cyberfall", "$work\cache" | Out-Null
Copy-Item "$repo\mod\r6\scripts\Cyberfall\*.reds" "$work\scripts\Cyberfall\"
New-Item -ItemType Directory -Force "$work\scripts\Codeware" | Out-Null
Copy-Item "$game\red4ext\plugins\Codeware\Scripts\*.reds" "$work\scripts\Codeware\"
Copy-Item "$game\r6\cache\final.redscripts" "$work\cache\"
$output = & "$game\engine\tools\scc.exe" -compile "$work\scripts" -customCacheDir "$work\cache" 2>&1 | Out-String
Remove-Item -Recurse -Force $work
Write-Host $output
if ($output -match '\[ERROR' -or $output -notmatch 'Compilation complete') { exit 1 }
if ($output -match '\[WARN') { Write-Host 'Compiled with warnings.' }
