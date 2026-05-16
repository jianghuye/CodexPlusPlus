use codex_plus_core::install::{
    InstallOptions, app_bundle_names, build_macos_app_bundle, build_uninstall_shortcut_script,
    build_windows_shortcut_script, shortcut_names,
};

#[test]
fn windows_shortcut_script_contains_silent_and_manager_entrypoints() {
    let options = InstallOptions {
        install_root: Some("C:/Users/A/Desktop".into()),
        launcher_path: Some("C:/Tools/codex-plus-plus.exe".into()),
        manager_path: Some("C:/Tools/codex-plus-plus-manager.exe".into()),
        remove_owned_data: false,
    };

    let script = build_windows_shortcut_script(&options);

    assert!(script.contains("Codex++.lnk"));
    assert!(script.contains("Codex++ 管理工具.lnk"));
    assert!(script.contains("codex-plus-plus.exe"));
    assert!(script.contains("codex-plus-plus-manager.exe"));
    assert!(
        script.contains("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CodexPlusPlus")
    );
    assert!(!script.contains("codex_session_delete"));
}

#[test]
fn windows_uninstall_script_removes_both_entrypoints() {
    let options = InstallOptions {
        install_root: Some("C:/Users/A/Desktop".into()),
        launcher_path: None,
        manager_path: None,
        remove_owned_data: true,
    };

    let script = build_uninstall_shortcut_script(&options);

    assert!(script.contains("Codex++.lnk"));
    assert!(script.contains("Codex++ 管理工具.lnk"));
    assert!(script.contains(".codex-session-delete"));
}

#[test]
fn macos_bundle_metadata_contains_silent_and_manager_apps() {
    let options = InstallOptions {
        install_root: Some("/Applications".into()),
        launcher_path: Some("/opt/Codex++/codex-plus-plus".into()),
        manager_path: Some("/opt/Codex++/codex-plus-plus-manager".into()),
        remove_owned_data: false,
    };

    let silent = build_macos_app_bundle(&options, false);
    let manager = build_macos_app_bundle(&options, true);

    assert!(silent.app_path.ends_with("Codex++.app"));
    assert!(manager.app_path.ends_with("Codex++ 管理工具.app"));
    assert!(silent.info_plist.contains("<string>Codex++</string>"));
    assert!(
        manager
            .info_plist
            .contains("<string>Codex++ 管理工具</string>")
    );
    assert!(silent.launch_script.contains("codex-plus-plus"));
    assert!(manager.launch_script.contains("codex-plus-plus-manager"));
}

#[test]
fn installer_exports_expected_two_entrypoint_names() {
    assert_eq!(shortcut_names(), ("Codex++.lnk", "Codex++ 管理工具.lnk"));
    assert_eq!(app_bundle_names(), ("Codex++.app", "Codex++ 管理工具.app"));
}
