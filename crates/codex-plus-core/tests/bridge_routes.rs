use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use codex_plus_core::models::{DeleteResult, DeleteStatus, ExportResult, ExportStatus, SessionRef};
use codex_plus_core::routes::{
    BridgeContext, BridgeDataService, BridgeRuntimeService, BridgeSettingsService,
    handle_bridge_request,
};
use codex_plus_core::settings::BackendSettings;
use serde_json::{Value, json};

#[tokio::test]
async fn bridge_routes_cover_all_current_paths() {
    let ctx = test_context();

    let cases = [
        ("/settings/get", json!({})),
        ("/settings/set", json!({"providerSyncEnabled": true})),
        ("/user-scripts/list", json!({})),
        ("/user-scripts/set-enabled", json!({"enabled": false})),
        (
            "/user-scripts/set-script-enabled",
            json!({"key": "user:a.js", "enabled": false}),
        ),
        ("/user-scripts/reload", json!({})),
        ("/devtools/open", json!({})),
        ("/backend/status", json!({})),
        ("/backend/repair", json!({})),
        ("/ads", json!({})),
        ("/delete", json!({"session_id": "s1", "title": "First"})),
        ("/undo", json!({"undo_token": "undo-1"})),
        (
            "/export-markdown",
            json!({"session_id": "s1", "title": "First"}),
        ),
        ("/archived-thread", json!({"title": "Archived"})),
        (
            "/move-thread-workspace",
            json!({"session_id": "s1", "title": "First", "target_cwd": "/new"}),
        ),
        (
            "/thread-sort-key",
            json!({"session_id": "s1", "title": "First"}),
        ),
        (
            "/thread-sort-keys",
            json!({"sessions": [{"session_id": "s1", "title": "First"}]}),
        ),
    ];

    for (path, payload) in cases {
        let result = handle_bridge_request(ctx.clone(), path, payload).await;
        assert_ne!(
            result["message"], "Unknown bridge path",
            "{path} should be routed"
        );
    }
}

#[tokio::test]
async fn unknown_bridge_path_matches_python_shape_with_empty_session_id() {
    let result = handle_bridge_request(
        test_context(),
        "/missing",
        json!({"session_id": "should-not-leak"}),
    )
    .await;

    assert_eq!(
        result,
        json!({
            "status": "failed",
            "session_id": "",
            "message": "Unknown bridge path"
        })
    );
}

#[tokio::test]
async fn settings_routes_use_settings_service() {
    let ctx = test_context();

    let updated = handle_bridge_request(
        ctx.clone(),
        "/settings/set",
        json!({"providerSyncEnabled": true, "cliWrapperApiKeyEnv": ""}),
    )
    .await;
    let loaded = handle_bridge_request(ctx, "/settings/get", json!({})).await;

    assert_eq!(updated["providerSyncEnabled"], true);
    assert_eq!(updated["cliWrapperApiKeyEnv"], "CUSTOM_OPENAI_API_KEY");
    assert_eq!(loaded, updated);
}

#[tokio::test]
async fn runtime_routes_keep_user_script_inventory_shape() {
    let ctx = test_context();

    let listed = handle_bridge_request(ctx.clone(), "/user-scripts/list", json!({})).await;
    let global = handle_bridge_request(
        ctx.clone(),
        "/user-scripts/set-enabled",
        json!({"enabled": false}),
    )
    .await;
    let script = handle_bridge_request(
        ctx.clone(),
        "/user-scripts/set-script-enabled",
        json!({"key": "user:a.js", "enabled": false}),
    )
    .await;
    let reloaded = handle_bridge_request(ctx, "/user-scripts/reload", json!({})).await;

    assert_eq!(listed["enabled"], true);
    assert_eq!(listed["scripts"][0]["key"], "builtin:demo.js");
    assert_eq!(global["enabled"], false);
    assert_eq!(script["scripts"][1]["enabled"], false);
    assert_eq!(reloaded["reloaded"], true);
    assert_eq!(reloaded["scripts"][0]["key"], "builtin:demo.js");
}

#[tokio::test]
async fn runtime_status_devtools_repair_and_ads_routes_are_dispatched() {
    let ctx = test_context();

    assert_eq!(
        handle_bridge_request(ctx.clone(), "/devtools/open", json!({})).await,
        json!({"status": "ok", "opened": true})
    );
    assert_eq!(
        handle_bridge_request(ctx.clone(), "/backend/status", json!({})).await,
        json!({"status": "ok", "message": "后端已连接"})
    );
    assert_eq!(
        handle_bridge_request(ctx.clone(), "/backend/repair", json!({})).await,
        json!({"status": "ok", "message": "后端已修复"})
    );
    assert_eq!(
        handle_bridge_request(ctx, "/ads", json!({})).await,
        json!({"version": 1, "ads": [{"id": "runtime-ad"}]})
    );
}

#[tokio::test]
async fn data_routes_forward_payloads_to_data_service() {
    let ctx = test_context();

    assert_eq!(
        handle_bridge_request(
            ctx.clone(),
            "/delete",
            json!({"session_id": "s1", "title": "First"}),
        )
        .await["undo_token"],
        "undo-s1"
    );
    assert_eq!(
        handle_bridge_request(ctx.clone(), "/undo", json!({"undo_token": "undo-s1"})).await,
        json!({
            "status": "undone",
            "session_id": "s1",
            "message": "undone",
            "undo_token": "undo-s1",
            "backup_path": null
        })
    );
    assert_eq!(
        handle_bridge_request(
            ctx.clone(),
            "/export-markdown",
            json!({"session_id": "s1", "title": "First"}),
        )
        .await["filename"],
        "First.md"
    );
    assert_eq!(
        handle_bridge_request(
            ctx.clone(),
            "/archived-thread",
            json!({"title": "Archived"})
        )
        .await,
        json!({"session_id": "archived-1", "title": "Archived"})
    );
    assert_eq!(
        handle_bridge_request(
            ctx.clone(),
            "/move-thread-workspace",
            json!({"session_id": "s1", "title": "First", "target_cwd": "/new"}),
        )
        .await,
        json!({"status": "moved", "session_id": "s1", "target_cwd": "/new"})
    );
    assert_eq!(
        handle_bridge_request(
            ctx.clone(),
            "/thread-sort-key",
            json!({"session_id": "s1", "title": "First"}),
        )
        .await,
        json!({"status": "ok", "session_id": "s1", "updated_at": 123})
    );
    assert_eq!(
        handle_bridge_request(
            ctx,
            "/thread-sort-keys",
            json!({"sessions": [{"session_id": "s1", "title": "First"}, null, {"session_id": "s2"}]}),
        )
        .await,
        json!({"status": "ok", "sort_keys": [{"session_id": "s1"}, {"session_id": "s2"}]})
    );
}

fn test_context() -> BridgeContext {
    BridgeContext::new(
        Arc::new(FakeSettings::default()),
        Arc::new(FakeRuntime::default()),
        Arc::new(FakeData::default()),
    )
}

#[derive(Default)]
struct FakeSettings {
    settings: Mutex<BackendSettings>,
}

#[async_trait]
impl BridgeSettingsService for FakeSettings {
    async fn get_settings(&self) -> anyhow::Result<BackendSettings> {
        Ok(self.settings.lock().unwrap().clone())
    }

    async fn set_settings(&self, payload: Value) -> anyhow::Result<BackendSettings> {
        let current = self.settings.lock().unwrap().clone();
        let mut raw = serde_json::to_value(current).unwrap();
        let raw = raw.as_object_mut().unwrap();
        if let Some(value) = payload.get("providerSyncEnabled").and_then(Value::as_bool) {
            raw.insert("providerSyncEnabled".to_string(), json!(value));
        }
        if let Some(value) = payload.get("cliWrapperApiKeyEnv").and_then(Value::as_str) {
            raw.insert(
                "cliWrapperApiKeyEnv".to_string(),
                json!(if value.is_empty() {
                    "CUSTOM_OPENAI_API_KEY"
                } else {
                    value
                }),
            );
        }
        let updated: BackendSettings = serde_json::from_value(Value::Object(raw.clone())).unwrap();
        *self.settings.lock().unwrap() = updated.clone();
        Ok(updated)
    }
}

struct FakeRuntime {
    enabled: Mutex<bool>,
    script_enabled: Mutex<bool>,
}

impl Default for FakeRuntime {
    fn default() -> Self {
        Self {
            enabled: Mutex::new(true),
            script_enabled: Mutex::new(true),
        }
    }
}

#[async_trait]
impl BridgeRuntimeService for FakeRuntime {
    async fn user_script_inventory(&self) -> anyhow::Result<Value> {
        Ok(self.inventory(false))
    }

    async fn set_user_scripts_enabled(&self, enabled: bool) -> anyhow::Result<Value> {
        *self.enabled.lock().unwrap() = enabled;
        Ok(self.inventory(false))
    }

    async fn set_user_script_enabled(&self, key: String, enabled: bool) -> anyhow::Result<Value> {
        assert_eq!(key, "user:a.js");
        *self.script_enabled.lock().unwrap() = enabled;
        Ok(self.inventory(false))
    }

    async fn reload_user_scripts(&self) -> anyhow::Result<Value> {
        Ok(self.inventory(true))
    }

    async fn open_devtools(&self) -> anyhow::Result<Value> {
        Ok(json!({"status": "ok", "opened": true}))
    }

    async fn backend_status(&self) -> anyhow::Result<Value> {
        Ok(json!({"status": "ok", "message": "后端已连接"}))
    }

    async fn repair_backend(&self) -> anyhow::Result<Value> {
        Ok(json!({"status": "ok", "message": "后端已修复"}))
    }

    async fn ads(&self) -> anyhow::Result<Value> {
        Ok(json!({"version": 1, "ads": [{"id": "runtime-ad"}]}))
    }
}

impl FakeRuntime {
    fn inventory(&self, reloaded: bool) -> Value {
        json!({
            "enabled": *self.enabled.lock().unwrap(),
            "reloaded": reloaded,
            "scripts": [
                {"key": "builtin:demo.js", "name": "demo.js", "enabled": true},
                {"key": "user:a.js", "name": "a.js", "enabled": *self.script_enabled.lock().unwrap()}
            ]
        })
    }
}

struct FakeData;

impl Default for FakeData {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl BridgeDataService for FakeData {
    async fn delete(&self, session: SessionRef) -> anyhow::Result<DeleteResult> {
        Ok(DeleteResult {
            status: DeleteStatus::LocalDeleted,
            session_id: session.session_id.clone(),
            message: format!("deleted {}", session.title),
            undo_token: Some(format!("undo-{}", session.session_id)),
            backup_path: None,
        })
    }

    async fn undo(&self, undo_token: String) -> anyhow::Result<DeleteResult> {
        Ok(DeleteResult {
            status: DeleteStatus::Undone,
            session_id: "s1".to_string(),
            message: "undone".to_string(),
            undo_token: Some(undo_token),
            backup_path: None,
        })
    }

    async fn export_markdown(&self, session: SessionRef) -> anyhow::Result<ExportResult> {
        Ok(ExportResult {
            status: ExportStatus::Exported,
            session_id: session.session_id,
            message: "exported".to_string(),
            filename: Some("First.md".to_string()),
            markdown: Some("# First\n".to_string()),
        })
    }

    async fn find_archived_thread_by_title(
        &self,
        title: String,
    ) -> anyhow::Result<Option<SessionRef>> {
        Ok(Some(SessionRef {
            session_id: "archived-1".to_string(),
            title,
        }))
    }

    async fn move_thread_workspace(
        &self,
        session: SessionRef,
        target_cwd: String,
    ) -> anyhow::Result<Value> {
        Ok(json!({"status": "moved", "session_id": session.session_id, "target_cwd": target_cwd}))
    }

    async fn thread_sort_key(&self, session: SessionRef) -> anyhow::Result<Value> {
        Ok(json!({"status": "ok", "session_id": session.session_id, "updated_at": 123}))
    }

    async fn thread_sort_keys(&self, sessions: Vec<SessionRef>) -> anyhow::Result<Value> {
        Ok(json!({
            "status": "ok",
            "sort_keys": sessions
                .into_iter()
                .map(|session| json!({"session_id": session.session_id}))
                .collect::<Vec<_>>()
        }))
    }
}
