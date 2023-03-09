use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub assets: AssetOptions,
    pub server: ServerOptions
}

impl ServerConfig {
    pub fn new() -> ServerConfig {
        ServerConfig {
            assets: AssetOptions {
                android: StorageOptions {
                    assetbundles: String::from("DownloadOutput/Android"),
                    manifests: String::from("manifests/Android")
                },
                ios: StorageOptions {
                    assetbundles: String::from("DownloadOutput/iOS"),
                    manifests: String::from("manifests/iOS")
                }
            },
            server: ServerOptions {
                port: 3000,
                https: HttpsOptions {
                    enabled: false,
                    cert: String::new(),
                    key: String::new()
                }
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetOptions {
    pub android: StorageOptions,
    pub ios: StorageOptions
}

#[derive(Serialize, Deserialize)]
pub struct StorageOptions {
    pub assetbundles: String,
    pub manifests: String
}

#[derive(Serialize, Deserialize)]
pub struct ServerOptions {
    pub port: u32,
    pub https: HttpsOptions
}

#[derive(Serialize, Deserialize)]
pub struct HttpsOptions {
    pub enabled: bool,
    pub cert: String,
    pub key: String,
}