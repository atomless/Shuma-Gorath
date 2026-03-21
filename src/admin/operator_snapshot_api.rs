use serde_json::json;
use spin_sdk::http::{Request, Response};

pub(crate) fn handle_admin_operator_snapshot<S>(_req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    match crate::observability::hot_read_projection::load_operator_snapshot_hot_read(
        store, "default",
    ) {
        Some(snapshot) => {
            let body =
                serde_json::to_string(&snapshot.payload).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
        None => {
            let body = serde_json::to_string(&json!({
                "schema_version": crate::observability::operator_snapshot::OPERATOR_SNAPSHOT_SCHEMA_VERSION,
                "error": "operator_snapshot_not_materialized",
                "message": "Operator snapshot is not materialized yet. Wait for the next telemetry refresh cycle."
            }))
            .unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(503)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
    }
}
