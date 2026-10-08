param(
    [string]$InstallDirectory = "$env:LOCALAPPDATA\Programs\SeelenUI-Manual",
    [switch]$Configure
)
$ErrorActionPreference = 'Stop'
$repoDirectory = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$buildDirectory = Join-Path $repoDirectory 'target\debug'
$requiredFiles = @('seelen-ui.exe', 'slu-service.exe', 'slu.exe', 'sluhk.dll', 'SHA256SUMS', 'SHA256SUMS.sig')
foreach ($name in $requiredFiles) {
    if (!(Test-Path -LiteralPath (Join-Path $buildDirectory $name))) {
        throw "Missing build artifact: $name. Build with the manual-launch and custom-protocol features first."
    }
}
if (Get-Process seelen-ui,slu-service -ErrorAction SilentlyContinue) {
    throw 'Exit Seelen UI before installing this build.'
}

New-Item -ItemType Directory -Path $InstallDirectory -Force | Out-Null
foreach ($name in $requiredFiles) {
    Copy-Item -LiteralPath (Join-Path $buildDirectory $name) -Destination $InstallDirectory -Force
}
$staticDirectory = Join-Path $InstallDirectory 'static'
New-Item -ItemType Directory -Path $staticDirectory -Force | Out-Null
Get-ChildItem -LiteralPath (Join-Path $repoDirectory 'src\static') |
    Copy-Item -Destination $staticDirectory -Recurse -Force
Copy-Item -LiteralPath (Join-Path $repoDirectory 'LICENSE') -Destination $InstallDirectory -Force
Copy-Item -LiteralPath (Join-Path $repoDirectory 'documentation\local-monitor-work.md') -Destination $InstallDirectory -Force

$settingsDirectory = Join-Path $env:APPDATA 'com.seelen.seelen-ui'
$settingsPath = Join-Path $settingsDirectory 'settings.json'
New-Item -ItemType Directory -Path $settingsDirectory -Force | Out-Null
if ($Configure -or !(Test-Path -LiteralPath $settingsPath)) {
if (Test-Path -LiteralPath $settingsPath) {
    Copy-Item -LiteralPath $settingsPath -Destination ($settingsPath + '.before-manual-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.bak')
    $settings = Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json -AsHashtable
} else {
    $settings = @{}
}
if (!$settings.ContainsKey('byWidget')) { $settings.byWidget = @{} }
foreach ($widgetId in @('@seelen/weg','@seelen/fancy-toolbar','@seelen/window-manager','@seelen/wallpaper-manager','@seelen/task-switcher','@seelen/workspaces-viewer','@seelen/apps-menu')) {
    if (!$settings.byWidget.ContainsKey($widgetId)) { $settings.byWidget[$widgetId] = @{} }
}
$settings.byWidget['@seelen/fancy-toolbar'].enabled = $false
$settings.byWidget['@seelen/window-manager'].enabled = $false
$settings.byWidget['@seelen/wallpaper-manager'].enabled = $false
$settings.byWidget['@seelen/task-switcher'].enabled = $false
$settings.byWidget['@seelen/workspaces-viewer'].enabled = $false
$settings.byWidget['@seelen/apps-menu'].enabled = $false
$dock = $settings.byWidget['@seelen/weg']
$dock.enabled = $true
$dock.position = 'Bottom'
$dock.mode = 'MinContent'
$dock.hideMode = 'Always'
$dock.size = 32
$dock.padding = 4
$dock.margin = 4
$dock.temporalItemsVisibility = 'OnMonitor'
$dock.pinnedItemsVisibility = 'Always'
$dock.showWindowTitle = $false
$settings.backupSyncEnabled = $false
$settings.drpc = $false
$settings | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath $settingsPath -Encoding utf8
}

# Ordinary Start Menu entry, deliberately outside its Startup folder.
$programsDirectory = [Environment]::GetFolderPath('Programs')
$shortcutPath = Join-Path $programsDirectory 'Seelen UI (manual launch).lnk'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = Join-Path $InstallDirectory 'seelen-ui.exe'
$shortcut.WorkingDirectory = $InstallDirectory
$shortcut.Description = 'Seelen UI with monitor-specific pins and monitor memory. Manual launch only.'
$shortcut.Save()
Write-Output "Installed: $InstallDirectory"
Write-Output "Start Menu shortcut: $shortcutPath"
