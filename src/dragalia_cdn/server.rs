use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, header::HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router
};
use axum_server::tls_rustls::RustlsConfig;
use colored::Colorize;
use regex::{Captures, Regex};
use std::{
    path::PathBuf,
    net::SocketAddr,
    fs,
    sync::Arc
};
use std::sync::LazyLock;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt}
};

use super::config::ServerConfig;

pub async fn start_server() {
    let mut server_config: ServerConfig;
    let mut should_save_config = false;

    if std::path::Path::new("config.json").exists() {
        server_config = match fs::read_to_string("config.json") {
            Ok(file) => match serde_json::from_str(&file) {
                Ok(config) => config,
                Err(e) => {
                    panic!("Failed to deserialize config.json: {:?}", e);
                }
            }
            Err(e) => {
                panic!("Failed to read config.json: {:?}", e);
            }
        };

        if server_config.manifestpaths.is_empty() {
            let generated_config = ServerConfig::new();
            if !generated_config.manifestpaths.is_empty() {
                server_config.manifestpaths.extend(generated_config.manifestpaths);
            }

            should_save_config = true;
        }
    } else {
        server_config = ServerConfig::new();
        should_save_config = true;
    }

    if should_save_config {
        let serialized_config = serde_json::to_string_pretty(&server_config).unwrap();
        fs::write("config.json", serialized_config).unwrap_or_else(|e| {
            panic!("Failed to write new config to config.json: {:?}", e);
        });
        info!("Saved config.")
    }

    let shared_config = Arc::new(server_config);

    if shared_config.assetpaths.is_empty() {
        panic!("No asset folders configured. Please edit config.json to point to the location of your assets.");
    }

    if shared_config.manifestpaths.is_empty() {
        warn!("No manifest folders configured. The server will be unable to serve file lists for fresh downloads.");
    }

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), shared_config.port);

    let port = shared_config.port;
    let use_https = shared_config.ssl;
    let cert_path = shared_config.cert.clone();
    let key_path = shared_config.key.clone();

    let app = Router::new()
        .route("/info", get(get_info))
        .route("/dl/manifests/{*path}", get(get_manifest))
        .route("/dl/assetbundles/{*path}", get(get_assetbundle))
        .with_state(shared_config);


    if use_https {
        let tls_config = Some(RustlsConfig::from_pem_file(
            cert_path,
            key_path
        ).await.unwrap_or_else(|e| {
            panic!("Failed to load TLS config: {}", e);
        }));

        info!("Starting HTTPS server on port {}!", port);

        axum_server::bind_rustls(addr, tls_config.unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap_or_else(|e| {
                panic!("Failed to start HTTPS server: {:?}", e);
            });
    } else {
        info!("Starting HTTP server on port {}!", port);

        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap_or_else(|e| {
                panic!("Failed to start HTTP server: {:?}", e);
            });
    }

    info!("Server is shutting down...");
}

async fn get_info() -> &'static str {
    "omg cross-platform rust"
}

async fn get_assetbundle(headers: HeaderMap, State(state): State<Arc<ServerConfig>>, Path(path): Path<String>) -> impl IntoResponse {
    static ASSETBUNDLE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(Android|iOS)/.*([A-Z2-7=]{2})/([A-Z2-7=]{52})$").unwrap());

    if !ASSETBUNDLE_REGEX.is_match(&path) {
        return Response::builder().status(StatusCode::FORBIDDEN).body(Body::empty()).unwrap()
    }

    let captures = ASSETBUNDLE_REGEX.captures(&path).unwrap();

    if state.verbose {
        debug!("Fetching asset bundle: {}", path);
    }

    get_file_response(&state.assetpaths, &captures, headers, &path).await
}

async fn get_manifest(headers: HeaderMap, State(state): State<Arc<ServerConfig>>, Path(path): Path<String>) -> impl IntoResponse {
    static MANIFEST_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(Android|iOS)/([A-Za-z0-9]{1,16})/(assetbundle\.(?:(?:en_us|en_eu|zh_cn|zh_tw)\.)?manifest)$").unwrap());

    if !MANIFEST_REGEX.is_match(&path) {
        return Response::builder().status(StatusCode::FORBIDDEN).body(Body::empty()).unwrap()
    }

    let captures = MANIFEST_REGEX.captures(&path).unwrap();

    if state.verbose {
        debug!("Fetching manifest: {}", path);
    }

    get_file_response(&state.manifestpaths, &captures, headers, &path).await
}

async fn get_file_response(dirs: &Vec<String>, captures: &Captures<'_>, headers: HeaderMap, original_path: &String) -> Response {
    for path in dirs {
        let mut base_path = PathBuf::new();

        base_path.push(path);
        base_path.push(&captures[2]);
        base_path.push(&captures[3]);

        if base_path.exists() {
            return match read_file_into_response(base_path).await {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to read found file: {:?}", e);
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap()
                }
            };
        }
    }

    info!("Failed to find file: {} - redirecting to API server", original_path);

    let api_header = match headers.get("reliable_token") {
        Some(val) => {val.to_str().unwrap_or("")}
        None => {
            error!("Missing reliable_token header. Please upgrade DragaliPatch to the latest version.");
            ""
        }
    };

    if !api_header.is_empty() {
        Redirect::permanent(&(String::from(api_header) + original_path)).into_response()
    } else {
        warn!("Could not find file for request path {}.", &captures[0]);

        Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap()
    }
}

async fn read_file_into_response(path: PathBuf) -> Result<Response, io::Error> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str("application/octet-stream")
                .unwrap()
        )
        .body(Body::from(buffer))
        .unwrap())
}
