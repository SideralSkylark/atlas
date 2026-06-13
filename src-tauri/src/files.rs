use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use std::fs;
use syntect::html::highlighted_html_for_string;
use tauri::{command, AppHandle, Runtime};

use crate::{repos, state::AppState};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RenderedFile {
    pub content: String,
    pub file_type: String, // "markdown", "code", "html", "plain"
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub relative_path: String,
    pub is_dir: bool,
}

#[command]
pub fn list_files<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    relative_path: String,
) -> Vec<FileEntry> {
    let mut files = Vec::new();
    let root = repos::repo_path(&app, &repo_id);
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
pub fn read_raw_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    relative_path: String,
) -> Result<String, String> {
    let path = repos::repo_path(&app, &repo_id).join(&relative_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
pub fn write_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    relative_path: String,
    content: String,
) -> Result<(), String> {
    let path = repos::repo_path(&app, &repo_id).join(&relative_path);
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[command]
pub fn render_file<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    repo_id: String,
    relative_path: String,
) -> Result<RenderedFile, String> {
    let path = repos::repo_path(&app, &repo_id).join(&relative_path);

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

#[command]
pub fn search_files<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    query: String,
) -> Vec<SearchResult> {
    let root = repos::repo_path(&app, &repo_id);
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
