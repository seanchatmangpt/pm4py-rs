//! A2A integration tests — real in-process Axum HTTP server per test.
//! Gate: only compiled/run with `--features a2a`.
//! Run: cargo test --features a2a a2a_integration

#[cfg(feature = "a2a")]
mod a2a_integration {
    use axum::Router;
    use reqwest::Client;
    use serde_json::{json, Value};
    use tokio::net::TcpListener;

    async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
        use pm4py::a2a::handler::{a2a_agent_card, a2a_handler};
        let app = Router::new()
            .route("/", axum::routing::post(a2a_handler))
            .route(
                "/.well-known/agent-card.json",
                axum::routing::get(a2a_agent_card),
            );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);
        let handle = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (url, handle)
    }

    fn minimal_log() -> Value {
        json!({
            "attributes": {},
            "traces": [{"id": "c1", "attributes": {}, "events": [
                {"activity": "A", "timestamp": "2024-01-01T10:00:00Z", "resource": null, "attributes": {}},
                {"activity": "B", "timestamp": "2024-01-01T10:30:00Z", "resource": null, "attributes": {}}
            ]}]
        })
    }

    // ── Agent card tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_agent_card_200() {
        let (url, _h) = start_test_server().await;
        let r = reqwest::get(format!("{}/.well-known/agent-card.json", url))
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
    }

    #[tokio::test]
    async fn test_agent_card_name_pm4py_rust() {
        let (url, _h) = start_test_server().await;
        let card: Value = reqwest::get(format!("{}/.well-known/agent-card.json", url))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(card["name"].as_str().unwrap_or(""), "pm4py-rust");
    }

    #[tokio::test]
    async fn test_agent_card_10_skills() {
        let (url, _h) = start_test_server().await;
        let card: Value = reqwest::get(format!("{}/.well-known/agent-card.json", url))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let skills = card["skills"].as_array().expect("skills must be array");
        assert_eq!(skills.len(), 10, "must have exactly 10 skills");
    }

    #[tokio::test]
    async fn test_agent_card_has_version() {
        let (url, _h) = start_test_server().await;
        let card: Value = reqwest::get(format!("{}/.well-known/agent-card.json", url))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(
            card["protocolVersion"].is_string() || card["version"].is_string(),
            "card must have protocolVersion or version"
        );
    }

    // ── tasks/send tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_tasks_send_valid_state() {
        let (url, _h) = start_test_server().await;
        let body = json!({"jsonrpc":"2.0","id":1,"method":"tasks/send","params":{
            "id":"integ-state-001",
            "message":{"role":"user","parts":[{"type":"data","data":{
                "tool":"pm4py_statistics","args":{"event_log":minimal_log()}
            }}]}
        }});
        let r: Value = Client::new()
            .post(format!("{}/", url))
            .json(&body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let state = r["result"]["status"]["state"].as_str().unwrap_or("");
        assert!(
            matches!(state, "submitted" | "working" | "completed"),
            "unexpected state: {}",
            state
        );
    }

    #[tokio::test]
    async fn test_tasks_send_then_get() {
        let (url, _h) = start_test_server().await;
        let client = Client::new();
        let task_id = "integ-get-001";
        let send = json!({"jsonrpc":"2.0","id":1,"method":"tasks/send","params":{
            "id":task_id,"message":{"role":"user","parts":[{"type":"data","data":{
                "tool":"pm4py_statistics","args":{"event_log":minimal_log()}
            }}]}
        }});
        client
            .post(format!("{}/", url))
            .json(&send)
            .send()
            .await
            .unwrap();
        let get = json!({"jsonrpc":"2.0","id":2,"method":"tasks/get","params":{"id":task_id}});
        let r: Value = client
            .post(format!("{}/", url))
            .json(&get)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let returned_id = r["result"]["id"].as_str().unwrap_or("");
        let error_code = r["error"]["code"].as_i64().unwrap_or(0);
        assert!(
            returned_id == task_id || error_code == -32001,
            "must return task or TASK_NOT_FOUND. Got: {:?}",
            r
        );
    }

    #[tokio::test]
    async fn test_tasks_cancel() {
        let (url, _h) = start_test_server().await;
        let client = Client::new();
        let task_id = "integ-cancel-001";
        let send = json!({"jsonrpc":"2.0","id":1,"method":"tasks/send","params":{
            "id":task_id,"message":{"role":"user","parts":[{"type":"data","data":{
                "tool":"pm4py_statistics","args":{"event_log":minimal_log()}
            }}]}
        }});
        client
            .post(format!("{}/", url))
            .json(&send)
            .send()
            .await
            .unwrap();
        let cancel =
            json!({"jsonrpc":"2.0","id":2,"method":"tasks/cancel","params":{"id":task_id}});
        let r: Value = client
            .post(format!("{}/", url))
            .json(&cancel)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let state = r["result"]["status"]["state"].as_str();
        let ec = r["error"]["code"].as_i64().unwrap_or(0);
        assert!(
            state == Some("canceled") || state == Some("completed") || ec == -32001,
            "cancel must return canceled/completed/TASK_NOT_FOUND: {:?}",
            r
        );
    }

    // ── Error protocol tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_unknown_method_32601() {
        let (url, _h) = start_test_server().await;
        let body = json!({"jsonrpc":"2.0","id":1,"method":"tasks/unknown_xyz","params":{}});
        let r: Value = Client::new()
            .post(format!("{}/", url))
            .json(&body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(r["error"]["code"].as_i64().unwrap_or(0), -32601);
    }

    #[tokio::test]
    async fn test_bad_version_32600() {
        let (url, _h) = start_test_server().await;
        let body = json!({"jsonrpc":"1.0","id":1,"method":"tasks/send","params":{}});
        let r: Value = Client::new()
            .post(format!("{}/", url))
            .json(&body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(r["error"]["code"].as_i64().unwrap_or(0), -32600);
    }

    // ── Tool execution tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_pm4py_statistics_produces_artifacts() {
        let (url, _h) = start_test_server().await;
        let body = json!({"jsonrpc":"2.0","id":10,"method":"tasks/send","params":{
            "id":"integ-stats-001","message":{"role":"user","parts":[{"type":"data","data":{
                "tool":"pm4py_statistics","args":{"event_log":minimal_log()}
            }}]}
        }});
        let r: Value = Client::new()
            .post(format!("{}/", url))
            .json(&body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            r["result"]["status"]["state"].as_str().unwrap_or(""),
            "completed"
        );
        let arts = r["result"]["artifacts"]
            .as_array()
            .expect("must have artifacts");
        assert!(
            !arts.is_empty(),
            "statistics must produce at least one artifact"
        );
    }

    #[tokio::test]
    async fn test_pm4py_discover_alpha_produces_artifacts() {
        let (url, _h) = start_test_server().await;
        let body = json!({"jsonrpc":"2.0","id":11,"method":"tasks/send","params":{
            "id":"integ-disc-001","message":{"role":"user","parts":[{"type":"data","data":{
                "tool":"pm4py_discover_alpha","args":{"event_log":minimal_log()}
            }}]}
        }});
        let r: Value = Client::new()
            .post(format!("{}/", url))
            .json(&body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            r["result"]["status"]["state"].as_str().unwrap_or(""),
            "completed"
        );
        assert!(!r["result"]["artifacts"]
            .as_array()
            .expect("must have artifacts")
            .is_empty());
    }
}
