use std::fs;
use std::sync::Arc;
use axum::{body, Router};
use axum::body::Full;
use axum::body::Empty;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::{Deserialize, Serialize};


#[tokio::main]
async fn main() {

    let server_config: ServerConfig;

    if std::path::Path::new("config.json").exists() {
        server_config = match fs::read_to_string("config.json") {
            Ok(file) => serde_json::from_str(&file).unwrap(),
            Err(_) => ServerConfig {
                assetbundle_path: String::from("assetbundles"),
                manifest_path: String::from("manifests")
            }
        };
    } else {
        server_config = ServerConfig {
            assetbundle_path: String::from("assetbundles"),
            manifest_path: String::from("manifests")
        };

        let serialized_config = serde_json::to_string(&server_config).unwrap();

        fs::write("config.json", serialized_config).unwrap();
    }

    let shared_config = Arc::new(server_config);

    let app = Router::new()
        .route("/info", get(get_info))
        .route("/dl/assetbundles/*path", get(get_assetbundle))
        .with_state(shared_config);

    let bind_future = axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service());

    println!("started server!");

    bind_future
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct ServerConfig {
    assetbundle_path: String,
    manifest_path: String,
}

async fn get_info() -> &'static str {
    "yoyo from crossplatform rust"
}

async fn get_assetbundle(State(state): State<Arc<ServerConfig>>, Path(path): Path<String>) -> impl IntoResponse {
    if path.contains("../") {
        return Response::builder().status(StatusCode::FORBIDDEN).body(body::boxed(Empty::new())).unwrap()
    }

    let asset_path = path.trim_start_matches('/');
    let mut actual_path = String::new();
    actual_path.push_str(&state.assetbundle_path);
    actual_path.push('/');
    actual_path.push_str(asset_path);

    println!("checking file path {actual_path}");

    match fs::read(actual_path) {
        Ok(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str("application/octet-stream")
                    .unwrap()
            )
            .body(body::boxed(Full::from(file)))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap()
    }
}
