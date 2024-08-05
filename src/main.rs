use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    response::IntoResponse,
    Router,
};
use chrono::Utc;
use clap::Parser;
use lazy_static::lazy_static;
use prometheus::{Encoder, IntCounter, IntCounterVec, Registry, TextEncoder};
use serde_json::json;
use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};
use tower_http::trace::TraceLayer;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref TOTAL_REQUESTS: IntCounter =
        IntCounter::new("argus_total_requests", "Total number of requests")
            .expect("metric can be created");
    static ref REQUESTS_BY_METHOD: IntCounterVec = IntCounterVec::new(
        prometheus::opts!(
            "argus_requests_by_method",
            "Number of requests by HTTP method"
        ),
        &["method"]
    )
    .expect("metric can be created");
    static ref RESPONSES_BY_STATUS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!(
            "argus_responses_by_status",
            "Number of responses by HTTP status code"
        ),
        &["status"]
    )
    .expect("metric can be created");
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, env = "ARGUS_IP", default_value = "0.0.0.0")]
    listen_addr: String,

    #[arg(long, env = "ARGUS_PORT", default_value_t = 8080)]
    port: u16,

    #[arg(long, env = "ARGUS_RESPONSE_HEADERS", value_parser = parse_headers)]
    response_headers: Option<HashMap<String, String>>,

    #[arg(long, env = "ARGUS_RESPONSE_BODY")]
    response_body: Option<String>,

    #[arg(long, env = "ARGUS_RESPONSE_BODY_FILE")]
    response_body_file: Option<PathBuf>,

    #[arg(long, env = "ARGUS_FILTER_ROUTES", value_delimiter = ',')]
    filter_routes: Option<Vec<String>>,

    #[arg(long, env = "ARGUS_FILTER_METHODS", value_parser = parse_methods, value_delimiter = ',')]
    filter_methods: Option<Vec<Method>>,

    #[arg(long, env = "ARGUS_RESPONSE_STATUS", value_parser = parse_status_code)]
    response_status: Option<StatusCode>,

    #[arg(long, env = "ARGUS_DISABLE_METRICS")]
    disable_metrics: bool,
}

fn parse_headers(s: &str) -> Result<HashMap<String, String>, String> {
    s.split(',')
        .map(|pair| {
            let mut parts = pair.splitn(2, ':');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Ok((key.trim().to_string(), value.trim().to_string())),
                _ => Err(format!("Invalid header format: {}", pair)),
            }
        })
        .collect()
}

fn parse_methods(s: &str) -> Result<Method, String> {
    s.parse().map_err(|_| format!("Invalid HTTP method: {}", s))
}

fn parse_status_code(s: &str) -> Result<StatusCode, String> {
    u16::from_str(s)
        .map_err(|_| format!("Invalid status code: {}", s))
        .and_then(|code| {
            StatusCode::from_u16(code).map_err(|_| format!("Invalid status code: {}", code))
        })
}

struct AppState {
    args: Arc<Args>,
}

#[tokio::main]
async fn main() {
    let args = Arc::new(Args::parse());
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    if !args.disable_metrics {
        REGISTRY.register(Box::new(TOTAL_REQUESTS.clone())).unwrap();
        REGISTRY
            .register(Box::new(REQUESTS_BY_METHOD.clone()))
            .unwrap();
        REGISTRY
            .register(Box::new(RESPONSES_BY_STATUS.clone()))
            .unwrap();
    }

    let state = Arc::new(AppState { args: args.clone() });

    let mut app = Router::new()
        .fallback(handler)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    if !args.disable_metrics {
        app = app.route("/metrics", axum::routing::get(metrics_handler));
    }

    let addr = format!("{}:{}", args.listen_addr, args.port);
    tracing::info!("Listening on {}", addr);
    if args.disable_metrics {
        tracing::info!("Metrics are disabled");
    } else {
        tracing::info!("Metrics are enabled and accessible at /metrics");
    }

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(State(state): State<Arc<AppState>>, req: Request<Body>) -> impl IntoResponse {
    if !state.args.disable_metrics {
        TOTAL_REQUESTS.inc();
        REQUESTS_BY_METHOD
            .with_label_values(&[req.method().as_str()])
            .inc();
    }

    let (parts, body) = req.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);

    let should_log = match (&state.args.filter_routes, &state.args.filter_methods) {
        (Some(routes), Some(methods)) => {
            routes.iter().any(|r| parts.uri.path().starts_with(r))
                && methods.contains(&parts.method)
        }
        (Some(routes), None) => routes.iter().any(|r| parts.uri.path().starts_with(r)),
        (None, Some(methods)) => methods.contains(&parts.method),
        (None, None) => true,
    };

    if should_log {
        let log_entry = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "method": parts.method.to_string(),
            "uri": parts.uri.to_string(),
            "headers": format!("{:?}", parts.headers),
            "body": body_str,
        });

        // Log to stdout
        println!("{}", serde_json::to_string(&log_entry).unwrap());
    }

    let status = state.args.response_status.unwrap_or(StatusCode::OK);
    let mut response = axum::response::Response::builder().status(status);

    if let Some(headers) = &state.args.response_headers {
        for (key, value) in headers {
            response = response.header(key, value);
        }
    }

    let body = if let Some(file_path) = &state.args.response_body_file {
        tokio::fs::read_to_string(file_path)
            .await
            .unwrap_or_else(|_| "Error reading file".to_string())
    } else if let Some(body) = &state.args.response_body {
        body.clone() + "\n"
    } else {
        serde_json::to_string(&json!({"status": "ok"})).unwrap() + "\n"
    };

    if !state.args.disable_metrics {
        RESPONSES_BY_STATUS
            .with_label_values(&[status.as_str()])
            .inc();
    }

    response.body(body).unwrap().into_response()
}

async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, mime::TEXT_PLAIN.as_ref())],
        buffer,
    )
}
