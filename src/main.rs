mod server_config;

#[macro_use]
mod log;

mod timestamp;

use std::{fs, process};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use axum::{body, Router};
use axum::body::Full;
use axum::body::Empty;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use crate::server_config::ServerConfig;
use colored::Colorize;

#[tokio::main]
async fn main() {

    let server_config: ServerConfig;

    if std::path::Path::new("config.json").exists() {
        server_config = match fs::read_to_string("config.json") {
            Ok(file) => match serde_json::from_str(&file) {
                Ok(config) => config,
                Err(e) => {
                    error!("Failed to deserialize config.json: {:?}", e);
                    process::exit(1);
                }
            }
            Err(e) => {
                error!("Failed to read config.json: {:?}", e);
                process::exit(1);
            }
        };
    } else {
        server_config = ServerConfig::new();
        let serialized_config = serde_json::to_string_pretty(&server_config).unwrap();
        fs::write("config.json", serialized_config).unwrap_or_else(|e| {
            error!("Failed to write new config to config.json: {:?}", e);
            process::exit(1);
        })
    }

    let shared_config = Arc::new(server_config);

    if shared_config.locations.assetbundles.is_empty() {
        error!("No asset folders configured. Please edit config.json to point to the location of your assets.");
        process::exit(1);
    }

    if shared_config.locations.manifests.is_empty() {
        warn!("No manifest folders configured. The server will be unable to serve file lists for fresh downloads.");
    }

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), shared_config.server.port);

    let port = shared_config.server.port;
    let use_https = shared_config.server.https.enabled;
    let cert_path = shared_config.server.https.cert.clone();
    let key_path = shared_config.server.https.key.clone();

    let app = Router::new()
        .route("/info", get(get_info))
        .route("/dl/manifests/*path", get(get_manifest))
        .route("/dl/assetbundles/*path", get(get_assetbundle))
        .with_state(shared_config);


   if use_https {
       let tls_config = Some(RustlsConfig::from_pem_file(
           cert_path,
           key_path
       ).await.unwrap_or_else(|e| {
           error!("Failed to load TLS config: {}", e);
           process::exit(1);
       }));

       info!("Starting HTTPS server on port {}!", port);

       axum_server::bind_rustls(addr, tls_config.unwrap())
           .serve(app.into_make_service())
           .await
           .unwrap_or_else(|e| {
               error!("Failed to start HTTPS server: {:?}", e);
               process::exit(1);
           });
   } else {
       info!("Starting HTTP server on port {}!", port);

       axum_server::bind(addr)
           .serve(app.into_make_service())
           .await
           .unwrap_or_else(|e| {
               error!("Failed to start HTTP server: {:?}", e);
               process::exit(1);
           });
   }

    info!("Server is shutting down...");
}

async fn get_info() -> &'static str {
    "omg cross-platform rust"
}

async fn get_assetbundle(State(state): State<Arc<ServerConfig>>, Path(path): Path<String>) -> impl IntoResponse {
    lazy_static! {
        static ref ASSETBUNDLE_REGEX: Regex = Regex::new(r"^(Android|iOS)/.*([A-Z2-7=]{2})/([A-Z2-7=]{52})$").unwrap();
    }

    if !ASSETBUNDLE_REGEX.is_match(&path) {
        return Response::builder().status(StatusCode::FORBIDDEN).body(body::boxed(Empty::new())).unwrap()
    }

    let captures = ASSETBUNDLE_REGEX.captures(&path).unwrap();

    get_file_response(&state.locations.assetbundles, captures)
}

async fn get_manifest(State(state): State<Arc<ServerConfig>>, Path(path): Path<String>) -> impl IntoResponse {
    lazy_static! {
        static ref MANIFEST_REGEX: Regex = Regex::new(r"^(Android|iOS)/([A-Za-z0-9]{1,16})/(assetbundle\.(?:(?:en_us|en_eu|zh_cn|zh_tw)\.)?manifest)$").unwrap();
    }

    if !MANIFEST_REGEX.is_match(&path) {
        return Response::builder().status(StatusCode::FORBIDDEN).body(body::boxed(Empty::new())).unwrap()
    }

    let captures = MANIFEST_REGEX.captures(&path).unwrap();

    get_file_response(&state.locations.manifests, captures)
}

fn get_file_response(dirs: &Vec<String>, captures: Captures) -> Response {
    for path in dirs {
        let mut base_path = PathBuf::new();

        base_path.push(path);
        base_path.push(&captures[2]);
        base_path.push(&captures[3]);

//        debug!("Checking file path {}", base_path.display());

        if base_path.exists() {
            return return_file_or_not_found(base_path)
        }
    }

    warn!("Could not find file for request path {}.", &captures[0]);

    Response::builder().status(StatusCode::NOT_FOUND).body(body::boxed(Empty::new())).unwrap()
}

fn return_file_or_not_found(path: PathBuf) -> Response {
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
        Err(e) => {
            error!("Could not open found file: {}", e);

            Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap()
        }
    }
}
