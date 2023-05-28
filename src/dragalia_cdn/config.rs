use std::ffi::OsStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    // Path settings
    pub assetpaths: Vec<String>,

    #[serde(default = "Vec::new")]
    pub manifestpaths: Vec<String>,

    // Server settings
    pub port: u16,

    // HTTPS settings
    pub ssl: bool,
    pub key: String,
    pub cert: String,
}

impl ServerConfig {
    pub fn new() -> ServerConfig {
        let mut asset_paths = Vec::<String>::new();
        let mut manifest_paths = Vec::<String>::new();

        let dirs = std::fs::read_dir(".")
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();

        for dir in dirs {
            if !dir.is_dir() {
                continue;
            }

            let dir_str = String::from(dir.to_string_lossy());

            match dir.file_name().unwrap().to_str().unwrap() {
                "manifest" => manifest_paths.push(dir_str),
                "orchis" => manifest_paths.push(dir_str),
                "assetbundles" => asset_paths.push(dir_str),
                _ => {
                    let subdir_entries = dir.read_dir().unwrap();
                    for subdir_entry in subdir_entries {
                        let entry_data = subdir_entry.unwrap();
                        let filename = entry_data.file_name();
                        let dir_path = String::from(entry_data.path().to_string_lossy());

                        if filename == OsStr::new("Android")
                            || filename == OsStr::new("iOS") {
                            asset_paths.push(dir_path);
                            break;
                        } else if filename == OsStr::new("2A") {
                            asset_paths.push(dir_str);
                            break;
                        } else if filename == OsStr::new("y2XM6giU6zz56wCm")
                            || filename == OsStr::new("b1HyoeTFegeTexC0") {
                            manifest_paths.push(dir_str);
                            break;
                        }
                    }
                }
            }
        }

        ServerConfig {
            assetpaths: asset_paths,
            manifestpaths: manifest_paths,
            port: 3000,
            ssl: false,
            cert: String::new(),
            key: String::new()
        }
    }
}