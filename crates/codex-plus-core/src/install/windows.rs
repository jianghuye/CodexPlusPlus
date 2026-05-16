use std::path::Path;
use std::process::Command;

use super::{
    InstallOptions, MANAGER_BINARY, MANAGER_NAME, SILENT_BINARY, SILENT_NAME,
    install_root_or_default, option_or_current_exe,
};

pub fn build_install_shortcut_script(options: &InstallOptions) -> String {
    let install_root = ps_quote(&install_root_or_default(options).to_string_lossy());
    let launcher =
        ps_quote(&option_or_current_exe(&options.launcher_path, SILENT_BINARY).to_string_lossy());
    let manager =
        ps_quote(&option_or_current_exe(&options.manager_path, MANAGER_BINARY).to_string_lossy());
    let icon = ps_quote(&default_icon_path().to_string_lossy());
    let version = crate::version::VERSION;
    format!(
        r#"$InstallRoot = {install_root}
$LauncherPath = {launcher}
$ManagerPath = {manager}
$CodexPlusIcon = {icon}
New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null
$Shell = New-Object -ComObject WScript.Shell
$SilentShortcutPath = Join-Path $InstallRoot 'Codex++.lnk'
$SilentShortcut = $Shell.CreateShortcut($SilentShortcutPath)
$SilentShortcut.TargetPath = $LauncherPath
$SilentShortcut.Arguments = ''
$SilentShortcut.WorkingDirectory = Split-Path -Parent $LauncherPath
$SilentShortcut.Description = 'Launch Codex++ silently'
$SilentShortcut.IconLocation = $CodexPlusIcon
$SilentShortcut.Save()
$ManagerShortcutPath = Join-Path $InstallRoot 'Codex++ 管理工具.lnk'
$ManagerShortcut = $Shell.CreateShortcut($ManagerShortcutPath)
$ManagerShortcut.TargetPath = $ManagerPath
$ManagerShortcut.Arguments = ''
$ManagerShortcut.WorkingDirectory = Split-Path -Parent $ManagerPath
$ManagerShortcut.Description = 'Open Codex++ management tool'
$ManagerShortcut.IconLocation = $CodexPlusIcon
$ManagerShortcut.Save()
$LegacyUninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Codex++'
if (Test-Path $LegacyUninstallKey) {{ Remove-Item $LegacyUninstallKey -Force }}
$UninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexPlusPlus'
$UninstallCommand = '"' + $ManagerPath + '"'
New-Item -Path $UninstallKey -Force | Out-Null
Set-ItemProperty -Path $UninstallKey -Name DisplayName -Value 'Codex++'
Set-ItemProperty -Path $UninstallKey -Name DisplayVersion -Value '{version}'
Set-ItemProperty -Path $UninstallKey -Name Publisher -Value 'BigPizzaV3'
Set-ItemProperty -Path $UninstallKey -Name DisplayIcon -Value $CodexPlusIcon
Set-ItemProperty -Path $UninstallKey -Name InstallLocation -Value (Split-Path -Parent $ManagerPath)
Set-ItemProperty -Path $UninstallKey -Name UninstallString -Value $UninstallCommand
Set-ItemProperty -Path $UninstallKey -Name QuietUninstallString -Value $UninstallCommand"#
    )
}

pub fn build_uninstall_shortcut_script(options: &InstallOptions) -> String {
    let install_root = ps_quote(&install_root_or_default(options).to_string_lossy());
    let data_dir = ps_quote(&crate::paths::default_app_state_dir().to_string_lossy());
    let remove_data = if options.remove_owned_data {
        format!("if (Test-Path {data_dir}) {{ Remove-Item {data_dir} -Recurse -Force }}")
    } else {
        String::new()
    };
    format!(
        r#"$InstallRoot = {install_root}
$SilentShortcutPath = Join-Path $InstallRoot 'Codex++.lnk'
$ManagerShortcutPath = Join-Path $InstallRoot 'Codex++ 管理工具.lnk'
if (Test-Path $SilentShortcutPath) {{ Remove-Item $SilentShortcutPath -Force }}
if (Test-Path $ManagerShortcutPath) {{ Remove-Item $ManagerShortcutPath -Force }}
$LegacyUninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Codex++'
if (Test-Path $LegacyUninstallKey) {{ Remove-Item $LegacyUninstallKey -Force }}
$UninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexPlusPlus'
if (Test-Path $UninstallKey) {{ Remove-Item $UninstallKey -Force }}
{remove_data}"#
    )
}

#[cfg(windows)]
pub fn install_shortcuts(options: &InstallOptions) -> anyhow::Result<()> {
    run_powershell(&build_install_shortcut_script(options))
}

#[cfg(windows)]
pub fn uninstall_shortcuts(options: &InstallOptions) -> anyhow::Result<()> {
    run_powershell(&build_uninstall_shortcut_script(options))
}

#[cfg(not(windows))]
pub fn install_shortcuts(_options: &InstallOptions) -> anyhow::Result<()> {
    anyhow::bail!("Windows shortcuts are only supported on Windows")
}

#[cfg(not(windows))]
pub fn uninstall_shortcuts(_options: &InstallOptions) -> anyhow::Result<()> {
    anyhow::bail!("Windows shortcuts are only supported on Windows")
}

fn run_powershell(script: &str) -> anyhow::Result<()> {
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("PowerShell installer exited with {status}")
    }
}

fn default_icon_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|path| path.join("codex-plus-plus.ico"))
        .unwrap_or_else(|| std::path::PathBuf::from("codex-plus-plus.ico"))
}

fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[allow(dead_code)]
fn _entrypoint_names() -> (&'static str, &'static str) {
    (SILENT_NAME, MANAGER_NAME)
}
