use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager, Runtime};

use crate::atlas_keystore;

#[derive(Serialize, Deserialize, Default)]
pub struct Credentials {
    // this may cause the user to have many domain with diferent tokens
    // perfer user@domain or some other identifier ok the key side
    pub pats: HashMap<String, String>, // domain -> token
}

// only used in cases where its not an android app maybe find a cleaner way of doing this
pub fn get_credentials_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to get app data dir")
        .join("credentials.json")
}

pub async fn load_credentials<R: Runtime>(app: &AppHandle<R>) -> Result<Credentials, String> {
    #[cfg(target_os = "android")]
    {
        let aliases = atlas_keystore::list_secrets(app.clone()).await?;
        let mut pats = HashMap::new();
        for alias in aliases {
            if let Ok(token) = atlas_keystore::get_secret(app.clone(), alias.clone()).await {
                pats.insert(alias, token);
            }
        }
        Ok(Credentials { pats })
    }
    #[cfg(not(target_os = "android"))]
    {
        let path = get_credentials_path(app);
        if !path.exists() {
            return Ok(Credentials::default());
        }
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }
}

#[command]
pub async fn save_pat<R: Runtime>(
    app: AppHandle<R>,
    domain: String,
    token: String,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        atlas_keystore::store_secret(app, domain, token).await
    }
    #[cfg(not(target_os = "android"))]
    {
        let mut creds = load_credentials(&app).await?;
        creds.pats.insert(domain, token);
        let path = get_credentials_path(&app);
        let json = serde_json::to_string(&creds).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }
}

#[command]
pub async fn get_pats<R: Runtime>(app: AppHandle<R>) -> Result<HashMap<String, String>, String> {
    load_credentials(&app).await.map(|c| c.pats)
}

#[command]
pub async fn delete_pat<R: Runtime>(app: AppHandle<R>, domain: String) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        atlas_keystore::delete_secret(app, domain).await
    }
    #[cfg(not(target_os = "android"))]
    {
        let mut creds = load_credentials(&app).await?;
        creds.pats.remove(&domain);
        let path = get_credentials_path(&app);
        let json = serde_json::to_string(&creds).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }
}
