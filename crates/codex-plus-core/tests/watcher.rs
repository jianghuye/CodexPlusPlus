use codex_plus_core::watcher::{
    build_install_watcher_script, build_spawn_launcher_command,
    build_stop_launcher_processes_script, build_uninstall_watcher_script, cdp_listening,
    disable_watcher_at, enable_watcher_at, watcher_disabled_flag,
};

#[test]
fn cdp_listening_returns_true_for_bound_loopback_port() {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();

    assert!(cdp_listening(port));
}

#[test]
fn cdp_listening_returns_false_for_closed_port() {
    let port = {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.local_addr().unwrap().port()
    };

    assert!(!cdp_listening(port));
}

#[test]
fn watcher_enable_and_disable_toggle_flag() {
    let dir = tempfile::tempdir().unwrap();
    let flag = watcher_disabled_flag(dir.path());

    disable_watcher_at(dir.path()).unwrap();
    assert!(flag.exists());

    enable_watcher_at(dir.path()).unwrap();
    assert!(!flag.exists());
}

#[test]
fn stop_launcher_script_protects_current_process_ancestry() {
    let script = build_stop_launcher_processes_script();

    assert!(script.contains("CODEX_PLUS_PLUS_PID"));
    assert!(script.contains("ParentProcessId"));
    assert!(script.contains("$protect.Contains([int]$_.ProcessId)"));
    assert!(script.contains("codex-plus-plus.exe"));
    assert!(!script.contains("codex_session_delete"));
}

#[test]
fn watcher_install_script_registers_rust_launcher_at_logon() {
    let script = build_install_watcher_script("C:/Tools/codex-plus-plus.exe", 9333);

    assert!(script.contains("CodexPlusPlusWatcher"));
    assert!(script.contains("CodexPlusPlusWatcher.lnk"));
    assert!(script.contains("codex-plus-plus.exe"));
    assert!(script.contains("--debug-port 9333"));
    assert!(!script.contains("python"));
    assert!(!script.contains("codex_session_delete"));
}

#[test]
fn watcher_uninstall_script_removes_rust_watcher_processes() {
    let script = build_uninstall_watcher_script();

    assert!(script.contains("CodexPlusPlusWatcher"));
    assert!(script.contains("CodexPlusPlusWatcher.lnk"));
    assert!(script.contains("codex-plus-plus.exe"));
    assert!(!script.contains("codex_session_delete"));
}

#[test]
fn spawn_launcher_command_points_to_silent_binary_only() {
    let command = build_spawn_launcher_command("C:/Tools/codex-plus-plus.exe", 9444);

    assert_eq!(command[0], "C:/Tools/codex-plus-plus.exe");
    assert!(command.contains(&"--debug-port".to_string()));
    assert!(command.contains(&"9444".to_string()));
    assert!(!command.iter().any(|part| part.contains("manager")));
}
