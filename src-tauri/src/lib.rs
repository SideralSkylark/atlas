use git2::{
    build::RepoBuilder, CertificateCheckStatus, Cred, FetchOptions, PushOptions, RemoteCallbacks,
    Repository,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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

async fn load_credentials<R: Runtime>(app: &AppHandle<R>) -> Result<Credentials, String> {
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
async fn save_pat<R: Runtime>(app: AppHandle<R>, domain: String, token: String) -> Result<(), String> {
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
async fn get_pats<R: Runtime>(app: AppHandle<R>) -> Result<HashMap<String, String>, String> {
    load_credentials(&app).await.map(|c| c.pats)
}

#[command]
async fn delete_pat<R: Runtime>(app: AppHandle<R>, domain: String) -> Result<(), String> {
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

async fn make_callbacks<R: Runtime>(app: &AppHandle<R>, url: &str) -> Result<RemoteCallbacks<'static>, String> {
    let creds = load_credentials(app).await?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.certificate_check(|_, _| Ok(CertificateCheckStatus::CertificateOk));

    if let Some(domain) = get_domain_from_url(url) {
        if let Some(token) = creds.pats.get(&domain) {
            let token = token.clone();
            callbacks.credentials(move |_, _, _| Cred::userpass_plaintext("git", &token));
        }
    }

    Ok(callbacks)
}

#[command]
async fn clone_repo<R: Runtime>(app: AppHandle<R>, url: String) -> CloneResult {
    let name = repo_name_from_url(&url);
    let dest = repo_path(&app, &name);

    if dest.exists() {
        return CloneResult {
            success: false,
            message: format!("'{}' already exists.", name),
        };
    }

    let callbacks = match make_callbacks(&app, &url).await {
        Ok(c) => c,
        Err(e) => return CloneResult { success: false, message: e },
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
async fn git_pull<R: Runtime>(app: AppHandle<R>, repo_id: String) -> Result<CloneResult, String> {
    let path = repo_path(&app, &repo_id);

    let url = {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
        remote.url().unwrap_or("").to_string()
    };

    let callbacks = make_callbacks(&app, &url).await?;

    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let head = repo.head().map_err(|e| e.to_string())?;
    let shorthand = head.shorthand().ok_or("Detached HEAD")?;
    let refname = head.name().ok_or("Invalid HEAD name")?;

    remote
        .fetch(&[shorthand], Some(&mut fetch_options), None)
        .map_err(|e| e.to_string())?;

    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|e| e.to_string())?;
    let fetch_commit = repo
        .reference_to_annotated_commit(&fetch_head)
        .map_err(|e| e.to_string())?;

    let (analysis, _) = repo
        .merge_analysis(&[&fetch_commit])
        .map_err(|e| e.to_string())?;

    if analysis.is_fast_forward() {
        let mut reference = repo.find_reference(refname).map_err(|e| e.to_string())?;
        reference
            .set_target(fetch_commit.id(), "Fast-forward")
            .map_err(|e| e.to_string())?;
        repo.set_head(refname).map_err(|e| e.to_string())?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| e.to_string())?;

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
async fn git_push<R: Runtime>(app: AppHandle<R>, repo_id: String) -> Result<CloneResult, String> {
    let path = repo_path(&app, &repo_id);

    let url = {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
        remote.url().unwrap_or("").to_string()
    };

    let callbacks = make_callbacks(&app, &url).await?;

    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    let head = repo.head().map_err(|e| e.to_string())?;
    let refname = head.name().ok_or("Invalid HEAD name")?;

    remote
        .push(
            &[format!("{}:{}", refname, refname)],
            Some(&mut push_options),
        )
        .map_err(|e| e.to_string())?;

    Ok(CloneResult {
        success: true,
        message: "Pushed successfully".to_string(),
    })
}

#[derive(Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

#[command]
fn list_branches<R: Runtime>(app: AppHandle<R>, repo_id: String) -> Result<Vec<BranchInfo>, String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let branches = repo.branches(None).map_err(|e| e.to_string())?;

    let mut branch_list = Vec::new();
    for branch_res in branches {
        let (branch, branch_type) = branch_res.map_err(|e| e.to_string())?;
        let name = branch
            .name()
            .map_err(|e| e.to_string())?
            .ok_or("Invalid branch name")?
            .to_string();
        let is_current = branch.is_head();
        let is_remote = branch_type == git2::BranchType::Remote;

        branch_list.push(BranchInfo {
            name,
            is_current,
            is_remote,
        });
    }

    Ok(branch_list)
}

#[command]
fn create_branch<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    branch_name: String,
) -> Result<(), String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let head = repo.head().map_err(|e| e.to_string())?;
    let target = head.peel_to_commit().map_err(|e| e.to_string())?;
    repo.branch(&branch_name, &target, false)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
fn switch_branch<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    branch_name: String,
) -> Result<(), String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let (object, reference) = repo.revparse_ext(&branch_name).map_err(|e| e.to_string())?;

    repo.checkout_tree(&object, Some(git2::build::CheckoutBuilder::default().force()))
        .map_err(|e| e.to_string())?;

    if let Some(refname) = reference.and_then(|r| r.name().map(|n| n.to_string())) {
        repo.set_head(&refname).map_err(|e| e.to_string())?;
    } else {
        repo.set_head_detached(object.id()).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub date: i64,
}

#[command]
fn get_commit_history<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    limit: usize,
) -> Result<Vec<CommitInfo>, String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;

    let mut commits = Vec::new();
    for id in revwalk.take(limit) {
        let id = id.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(id).map_err(|e| e.to_string())?;

        commits.push(CommitInfo {
            hash: commit.id().to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            message: commit.message().unwrap_or("").to_string(),
            date: commit.time().seconds(),
        });
    }

    Ok(commits)
}

#[derive(Serialize, Deserialize)]
pub struct StatusEntry {
    pub path: String,
    pub status: String, // "Modified", "New", "Deleted", "Renamed", etc.
    pub staged: bool,
}

#[command]
fn get_status<R: Runtime>(app: AppHandle<R>, repo_id: String) -> Result<Vec<StatusEntry>, String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let mut status_options = git2::StatusOptions::new();
    status_options
        .include_untracked(true)
        .recurse_untracked_dirs(true);

    let statuses = repo
        .statuses(Some(&mut status_options))
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        let status_str = if status.is_index_new() || status.is_wt_new() {
            "New"
        } else if status.is_index_modified() || status.is_wt_modified() {
            "Modified"
        } else if status.is_index_deleted() || status.is_wt_deleted() {
            "Deleted"
        } else if status.is_index_renamed() || status.is_wt_renamed() {
            "Renamed"
        } else {
            "Other"
        };

        let staged = status.is_index_new()
            || status.is_index_modified()
            || status.is_index_deleted()
            || status.is_index_renamed();

        entries.push(StatusEntry {
            path,
            status: status_str.to_string(),
            staged,
        });
    }

    Ok(entries)
}

#[command]
fn stage_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    filepath: String,
) -> Result<(), String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;

    let full_path = path.join(&filepath);
    if full_path.exists() {
        index
            .add_path(std::path::Path::new(&filepath))
            .map_err(|e| e.to_string())?;
    } else {
        index
            .remove_path(std::path::Path::new(&filepath))
            .map_err(|e| e.to_string())?;
    }
    index.write().map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
fn unstage_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    filepath: String,
) -> Result<(), String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let head = repo.head().map_err(|e| e.to_string())?;
    let commit = head.peel_to_commit().map_err(|e| e.to_string())?;

    repo.reset_default(Some(commit.as_object()), &[filepath])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
fn commit_changes<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    message: String,
    author_name: String,
    author_email: String,
) -> Result<(), String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;
    let tree_id = index.write_tree().map_err(|e| e.to_string())?;
    let tree = repo.find_tree(tree_id).map_err(|e| e.to_string())?;

    let sig = git2::Signature::now(&author_name, &author_email).map_err(|e| e.to_string())?;

    let parent_commit = repo.head().and_then(|h| h.peel_to_commit()).ok();
    let parents = if let Some(ref p) = parent_commit {
        vec![p]
    } else {
        vec![]
    };

    repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
fn get_diff<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    filepath: Option<String>,
    staged: bool,
) -> Result<String, String> {
    let path = repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let mut diff_options = git2::DiffOptions::new();
    if let Some(ref f) = filepath {
        diff_options.pathspec(f);
    }

    let diff = if staged {
        let head = repo.head().and_then(|h| h.peel_to_tree()).ok();
        repo.diff_tree_to_index(
            head.as_ref(),
            Some(&repo.index().map_err(|e| e.to_string())?),
            Some(&mut diff_options),
        )
    } else {
        repo.diff_index_to_workdir(None, Some(&mut diff_options))
    }
    .map_err(|e| e.to_string())?;

    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let prefix = match line.origin() {
            '+' => "+",
            '-' => "-",
            ' ' => " ",
            _ => "",
        };
        diff_text.push_str(prefix);
        diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
        true
    })
    .map_err(|e| e.to_string())?;

    Ok(diff_text)
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
    state: tauri::State<'_, AppState>,
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
        let ps = &state.syntax_set;
        let ts = &state.theme_set;

        // Try to find syntax by extension
        let syntax = ps
            .find_syntax_by_extension(&extension)
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

mod atlas_keystore;

#[command]
fn read_raw_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    relative_path: String,
) -> Result<String, String> {
    let path = repo_path(&app, &repo_id).join(&relative_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
fn write_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    relative_path: String,
    content: String,
) -> Result<(), String> {
    let path = repo_path(&app, &repo_id).join(&relative_path);
    fs::write(&path, content).map_err(|e| e.to_string())
}

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
        .take(100)
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
    }

    results
}

pub struct AppState {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let base_path = get_base_path(app.handle());
            if !base_path.exists() {
                fs::create_dir_all(base_path).expect("failed to create repos directory");
            }
            app.manage(AppState {
                syntax_set: SyntaxSet::load_defaults_newlines(),
                theme_set: ThemeSet::load_defaults(),
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(atlas_keystore::init())
        .invoke_handler(tauri::generate_handler![
            clone_repo,
            delete_repo,
            list_repos,
            list_files,
            list_branches,
            create_branch,
            switch_branch,
            get_commit_history,
            get_status,
            stage_file,
            unstage_file,
            commit_changes,
            get_diff,
            read_raw_file,
            write_file,
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
