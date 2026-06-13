use git2::{build::RepoBuilder, FetchOptions};
use tauri::{command, AppHandle, Runtime};

use crate::git::auth;
use crate::repos;
use crate::repos::CloneResult;

#[command]
pub async fn clone_repo<R: Runtime>(app: AppHandle<R>, url: String) -> CloneResult {
    let name = repos::repo_name_from_url(&url);
    let dest = repos::repo_path(&app, &name);

    if dest.exists() {
        return CloneResult {
            success: false,
            message: format!("'{}' already exists.", name),
        };
    }

    let callbacks = match auth::make_callbacks(&app, &url).await {
        Ok(c) => c,
        Err(e) => {
            return CloneResult {
                success: false,
                message: e,
            }
        }
    };

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);

    match builder.clone(&url, &dest) {
        Ok(_) => CloneResult {
            success: true,
            message: format!("Cloned '{}'.", name),
        },
        Err(e) => CloneResult {
            success: false,
            message: e.message().to_string(),
        },
    }
}
