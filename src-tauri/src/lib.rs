use git2::{
    build::RepoBuilder, Cred, CertificateCheckStatus, FetchOptions, PushOptions, RemoteCallbacks, Repository,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize, Default)]
pub struct Credentials {
    pub pats: HashMap<String, String>, // domain -> token
}

fn get_credentials_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to get app data dir")
        .join("credentials.json")
}

fn load_credentials<R: Runtime>(app: &AppHandle<R>) -> Credentials {
    let path = get_credentials_path(app);
    if !path.exists() {
        return Credentials::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[command]
fn save_pat<R: Runtime>(app: AppHandle<R>, domain: String, token: String) -> Result<(), String> {
    let mut creds = load_credentials(&app);
    creds.pats.insert(domain, token);
    let path = get_credentials_path(&app);
    let json = serde_json::to_string(&creds).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[command]
fn get_pats<R: Runtime>(app: AppHandle<R>) -> HashMap<String, String> {
    load_credentials(&app).pats
}

#[command]
fn delete_pat<R: Runtime>(app: AppHandle<R>, domain: String) -> Result<(), String> {
    let mut creds = load_credentials(&app);
    creds.pats.remove(&domain);
    let path = get_credentials_path(&app);
    let json = serde_json::to_string(&creds).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

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

fn get_domain_from_url(url: &str) -> Option<String> {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?
        .split('/')
        .next()
        .map(|s| s.to_string())
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

    let domain = get_domain_from_url(&url);
    let creds = load_credentials(&app);

    let mut callbacks = RemoteCallbacks::new();
    callbacks.certificate_check(|_, _| Ok(CertificateCheckStatus::CertificateOk));
    
    if let Some(d) = domain {
        if let Some(token) = creds.pats.get(&d) {
            let token_clone = token.clone();
            callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                Cred::userpass_plaintext("git", &token_clone)
            });
        }
    }

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

#[command]
fn git_pull<R: Runtime>(app: AppHandle<R>, repo_id: String) -> Result<CloneResult, String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;

    let url = remote.url().unwrap_or("").to_string();
    let domain = get_domain_from_url(&url);
    let creds = load_credentials(&app);

    let mut callbacks = RemoteCallbacks::new();
    callbacks.certificate_check(|_, _| Ok(CertificateCheckStatus::CertificateOk));
    
    if let Some(d) = domain {
        if let Some(token) = creds.pats.get(&d) {
            let token_clone = token.clone();
            callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                Cred::userpass_plaintext("git", &token_clone)
            });
        }
    }

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    remote.fetch(&["main"], Some(&mut fetch_options), None).map_err(|e| e.to_string())?;

    let fetch_head = repo.find_reference("FETCH_HEAD").map_err(|e| e.to_string())?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).map_err(|e| e.to_string())?;

    let (analysis, _) = repo.merge_analysis(&[&fetch_commit]).map_err(|e| e.to_string())?;

    if analysis.is_fast_forward() {
        let refname = "refs/heads/main";
        let mut reference = repo.find_reference(refname).map_err(|e| e.to_string())?;
        reference.set_target(fetch_commit.id(), "Fast-forward").map_err(|e| e.to_string())?;
        repo.set_head(refname).map_err(|e| e.to_string())?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())).map_err(|e| e.to_string())?;
        
        Ok(CloneResult {
            success: true,
            message: "Pulled successfully (Fast-forward)".to_string(),
        })
    } else if analysis.is_up_to_date() {
        Ok(CloneResult {
            success: true,
            message: "Already up to date".to_string(),
        })
    } else {
        Err("Pull failed: Only fast-forward merges are supported for now.".to_string())
    }
}

#[command]
fn git_push<R: Runtime>(app: AppHandle<R>, repo_id: String) -> Result<CloneResult, String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;

    let url = remote.url().unwrap_or("").to_string();
    let domain = get_domain_from_url(&url);
    let creds = load_credentials(&app);

    let mut callbacks = RemoteCallbacks::new();
    callbacks.certificate_check(|_, _| Ok(CertificateCheckStatus::CertificateOk));
    
    if let Some(d) = domain {
        if let Some(token) = creds.pats.get(&d) {
            let token_clone = token.clone();
            callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                Cred::userpass_plaintext("git", &token_clone)
            });
        }
    }

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options)).map_err(|e| e.to_string())?;

    Ok(CloneResult {
        success: true,
        message: "Pushed successfully".to_string(),
    })
}

use pulldown_cmark::{html, Parser};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

#[derive(Serialize, Deserialize)]
pub struct RenderedFile {
    pub content: String,
    pub file_type: String, // "markdown", "code", "html", "plain"
}

#[command]
fn render_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    relative_path: String,
) -> Result<RenderedFile, String> {
    let path = repo_path(&app, &repo_id).join(&relative_path);

    if !path.exists() {
        return Err("File not found".to_string());
    }

    if path.is_dir() {
        return Err("Cannot render a directory".to_string());
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;

    if extension == "md" || extension == "markdown" {
        let parser = Parser::new(&content);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        Ok(RenderedFile {
            content: html_output,
            file_type: "markdown".to_string(),
        })
    } else if extension == "html" || extension == "htm" {
        Ok(RenderedFile {
            content,
            file_type: "html".to_string(),
        })
    } else {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        
        // Try to find syntax by extension
        let syntax = ps.find_syntax_by_extension(&extension)
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        let theme = &ts.themes["base16-ocean.dark"];
        
        match highlighted_html_for_string(&content, &ps, syntax, theme) {
            Ok(html) => Ok(RenderedFile {
                content: html,
                file_type: "code".to_string(),
            }),
            Err(_) => Ok(RenderedFile {
                content,
                file_type: "plain".to_string(),
            }),
        }
    }
}

use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub relative_path: String,
    pub is_dir: bool,
}

#[command]
fn search_files<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    query: String,
) -> Vec<SearchResult> {
    let root = repo_path(&app, &repo_id);
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_str().map(|s| s == ".git").unwrap_or(false))
        .flatten()
    {
        let name = entry.file_name().to_string_lossy();
        if name.to_lowercase().contains(&query_lower) {
            let path = entry.path();
            let rel_path = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            
            if rel_path.is_empty() {
                continue;
            }

            results.push(SearchResult {
                name: name.to_string(),
                relative_path: rel_path,
                is_dir: path.is_dir(),
            });
        }
        
        if results.len() > 100 {
            break;
        }
    }

    results
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
            read_file,
            render_file,
            search_files,
            save_pat,
            get_pats,
            delete_pat,
            git_pull,
            git_push
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
