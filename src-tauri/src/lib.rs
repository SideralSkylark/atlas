use git2::{build::RepoBuilder, CertificateCheckStatus, FetchOptions, RemoteCallbacks, Repository};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize)]
pub struct CloneResult {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct RepoInfo {
    pub id: String,
    pub name: String,
    pub branch: String,
}

fn get_base_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to get app data dir")
        .join("repos")
}

fn repo_path<R: Runtime>(app: &AppHandle<R>, repo_id: &str) -> PathBuf {
    get_base_path(app).join(repo_id)
}

/// Derives a repo folder name from a clone URL.
/// "https://github.com/user/my-repo.git" → "my-repo"
fn repo_name_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(url)
        .trim_end_matches(".git")
        .to_string()
}

#[command]
fn clone_repo<R: Runtime>(app: AppHandle<R>, url: String) -> CloneResult {
    let name = repo_name_from_url(&url);
    let dest = repo_path(&app, &name);

    if dest.exists() {
        return CloneResult {
            success: false,
            message: format!("'{}' already exists.", name),
        };
    }

    let mut callbacks = RemoteCallbacks::new();
    callbacks.certificate_check(|_, _| Ok(CertificateCheckStatus::CertificateOk));

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

#[command]
fn delete_repo<R: Runtime>(app: AppHandle<R>, repo_id: String) -> CloneResult {
    let path = repo_path(&app, &repo_id);
    match fs::remove_dir_all(path) {
        Ok(_) => CloneResult {
            success: true,
            message: "Repo deleted.".to_string(),
        },
        Err(e) => CloneResult {
            success: false,
            message: e.to_string(),
        },
    }
}

#[command]
fn list_repos<R: Runtime>(app: AppHandle<R>) -> Vec<RepoInfo> {
    let mut repos = Vec::new();
    let base = get_base_path(&app);

    if !base.exists() {
        return repos;
    }

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(repo) = Repository::open(&path) {
                    let branch = repo
                        .head()
                        .ok()
                        .and_then(|h| h.shorthand().map(|s| s.to_string()))
                        .unwrap_or_else(|| "detached".to_string());
                    repos.push(RepoInfo {
                        id: entry.file_name().to_string_lossy().to_string(),
                        name: entry.file_name().to_string_lossy().to_string(),
                        branch,
                    });
                }
            }
        }
    }

    repos
}

#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
}

#[command]
fn list_files<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    relative_path: String,
) -> Vec<FileEntry> {
    let mut files = Vec::new();
    let root = repo_path(&app, &repo_id);
    let path = if relative_path.is_empty() {
        root
    } else {
        root.join(&relative_path)
    };

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" {
                continue;
            }
            files.push(FileEntry {
                name,
                is_dir: p.is_dir(),
            });
        }
    }

    files.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    files
}

#[command]
fn read_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    relative_path: String,
) -> Result<String, String> {
    let path = repo_path(&app, &repo_id).join(relative_path);

    if !path.exists() {
        return Err("File not found".to_string());
    }

    if path.is_dir() {
        return Err("Cannot read a directory".to_string());
    }

    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let base_path = get_base_path(app.handle());
            if !base_path.exists() {
                fs::create_dir_all(base_path).expect("failed to create repos directory");
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            clone_repo,
            delete_repo,
            list_repos,
            list_files,
            read_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
