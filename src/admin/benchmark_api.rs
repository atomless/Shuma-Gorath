use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

pub(crate) fn handle_admin_benchmark_suite(req: &Request) -> Response {
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let body = serde_json::to_string(&crate::observability::benchmark_suite::benchmark_suite_v1())
        .unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub(crate) fn handle_admin_benchmark_results<S>(req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    match crate::observability::hot_read_projection::load_operator_snapshot_hot_read(
        store, "default",
    ) {
        Some(snapshot) => {
            let payload = snapshot.payload.benchmark_results.clone();
            let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
        None => {
            let body = serde_json::to_string(&json!({
                "schema_version": crate::observability::benchmark_results::BENCHMARK_RESULTS_SCHEMA_VERSION,
                "error": "benchmark_results_snapshot_missing",
                "message": "Benchmark results require an already-materialized operator snapshot."
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
