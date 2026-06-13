use std::fs;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use tauri::Manager;

pub mod atlas_keystore;
pub mod credentials;
pub mod files;
pub mod git;
pub mod repos;
pub mod state;

// Credentials
use credentials::{delete_pat, get_pats, save_pat};

// Files
use files::{list_files, read_raw_file, render_file, search_files, write_file};

// Git
use git::{
    branch::{create_branch, list_branches, switch_branch},
    clone::clone_repo,
    commit::{commit_changes, get_commit_history},
    remote::{git_pull, git_push},
    status::{get_diff, get_status, stage_file, unstage_file},
};

// Repositories
use repos::{delete_repo, list_repos};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let base_path = repos::get_base_path(app.handle());
            if !base_path.exists() {
                fs::create_dir_all(base_path).expect("failed to create repos directory");
            }
            app.manage(state::AppState {
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
