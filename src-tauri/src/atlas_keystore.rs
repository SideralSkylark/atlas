use serde::{Deserialize, Serialize};
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

#[cfg(target_os = "android")]
use tauri::plugin::PluginHandle;

#[cfg(target_os = "android")]
#[derive(Serialize)]
pub struct StoreSecretArgs {
    alias: String,
    value: String,
}

#[cfg(target_os = "android")]
#[derive(Serialize)]
struct AliasArgs {
    alias: String,
}

#[cfg(target_os = "android")]
#[derive(Deserialize)]
struct GetSecretResponse {
    value: String,
}

#[cfg(target_os = "android")]
#[derive(Deserialize)]
struct ListSecretsResponse {
    secrets: Vec<String>,
}

#[command]
pub async fn store_secret<R: Runtime>(
    app: AppHandle<R>,
    alias: String,
    value: String,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let handle = app.state::<PluginHandle<R>>();
        handle
            .run_mobile_plugin::<()>("store_secret", StoreSecretArgs { alias, value })
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, alias, value);
        Err("AtlasKeystore is only supported on Android".to_string())
    }
}

#[command]
pub async fn get_secret<R: Runtime>(app: AppHandle<R>, alias: String) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        let handle = app.state::<PluginHandle<R>>();
        handle
            .run_mobile_plugin::<GetSecretResponse>("get_secret", AliasArgs { alias })
            .map(|r| r.value)
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, alias);
        Err("AtlasKeystore is only supported on Android".to_string())
    }
}

#[command]
pub async fn delete_secret<R: Runtime>(app: AppHandle<R>, alias: String) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let handle = app.state::<PluginHandle<R>>();
        handle
            .run_mobile_plugin::<()>("delete_secret", AliasArgs { alias })
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, alias);
        Err("AtlasKeystore is only supported on Android".to_string())
    }
}

#[command]
pub async fn list_secrets<R: Runtime>(app: AppHandle<R>) -> Result<Vec<String>, String> {
    #[cfg(target_os = "android")]
    {
        let handle = app.state::<PluginHandle<R>>();
        handle
            .run_mobile_plugin::<ListSecretsResponse>("list_secrets", ())
            .map(|r| r.secrets)
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Err("AtlasKeystore is only supported on Android".to_string())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("atlas-keystore")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            {
                let handle = api.register_android_plugin("com.skylark.atlas", "AtlasKeystore")?;
                app.manage(handle);
            }
            let _ = (app, api);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            store_secret,
            get_secret,
            delete_secret,
            list_secrets
        ])
        .build()
}
