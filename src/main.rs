mod server_config;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use axum::{body, Router};
use axum::body::Full;
use axum::body::Empty;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use lazy_static::lazy_static;
use regex::Regex;
use crate::server_config::ServerConfig;


#[tokio::main]
async fn main() {
    let server_config: ServerConfig;

    if std::path::Path::new("config.json").exists() {
        server_config = match fs::read_to_string("config.json") {
            Ok(file) => serde_json::from_str(&file).unwrap(),
            Err(_) => ServerConfig::new()
        };
    } else {
        server_config = ServerConfig::new();

        let serialized_config = serde_json::to_string(&server_config).unwrap();

        fs::write("config.json", serialized_config).unwrap();
    }

    let shared_config = Arc::new(server_config);

    let app = Router::new()
        .route("/info", get(get_info))
        .route("/dl/manifests/*path", get(get_manifest))
        .route("/dl/assetbundles/*path", get(get_assetbundle))
        .with_state(shared_config);

    let bind_future = axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service());

    println!("started server!");

    bind_future
        .await
        .unwrap();
}

async fn get_info() -> &'static str {
    "omg crossplatform rust"
}

async fn get_assetbundle(State(state): State<Arc<ServerConfig>>, Path(path): Path<String>) -> impl IntoResponse {
    lazy_static! {
            static ref ASSETBUNDLE_REGEX: Regex = Regex::new(r"^(Android|iOS)/([A-Z2-7=]{2})/([A-Z2-7=]{52})$").unwrap();
    }

    if ASSETBUNDLE_REGEX.is_match(&path) {
        return Response::builder().status(StatusCode::FORBIDDEN).body(body::boxed(Empty::new())).unwrap()
    }

    let mut base_path = PathBuf::new();
    let captures = ASSETBUNDLE_REGEX.captures(&path).unwrap();

    if &captures[0] == "Android" {
        base_path.push(&state.assets.android.assetbundles);
    } else {
        base_path.push(&state.assets.ios.assetbundles);
    }

    base_path.push(&captures[1]);
    base_path.push(&captures[2]);

    println!("checking file path {}", base_path.display());

    get_file_result(base_path)
}

async fn get_manifest(State(state): State<Arc<ServerConfig>>, Path(path): Path<String>) -> impl IntoResponse {
    lazy_static! {
        static ref MANIFEST_REGEX: Regex = Regex::new(r"^(Android|iOS)/([A-Za-z0-9]{1,16})/(assetbundle\.(?:(?:en_us|en_eu|zh_cn|zh_tw)\.)?manifest)$").unwrap();
    }

    if MANIFEST_REGEX.is_match(&path) {
        return Response::builder().status(StatusCode::FORBIDDEN).body(body::boxed(Empty::new())).unwrap()
    }

    let mut base_path = PathBuf::new();
    let captures = MANIFEST_REGEX.captures(&path).unwrap();

    if &captures[0] == "Android" {
        base_path.push(&state.assets.android.manifests);
    } else {
        base_path.push(&state.assets.ios.manifests);
    }

    base_path.push(&captures[1]);
    base_path.push(&captures[2]);

    println!("checking file path {}", base_path.display());

    get_file_result(base_path)
}

fn get_file_result(path: PathBuf) -> Response {
    match fs::read(path) {
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
