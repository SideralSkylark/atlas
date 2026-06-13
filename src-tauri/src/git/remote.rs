use git2::{FetchOptions, PushOptions, Repository};
use tauri::{command, AppHandle, Runtime};

use crate::git::auth;
use crate::repos;
use crate::repos::CloneResult;

#[command]
pub async fn git_pull<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
) -> Result<CloneResult, String> {
    let path = repos::repo_path(&app, &repo_id);

    let url = {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
        remote.url().unwrap_or("").to_string()
    };

    let callbacks = auth::make_callbacks(&app, &url).await?;

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
pub async fn git_push<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
) -> Result<CloneResult, String> {
    let path = repos::repo_path(&app, &repo_id);

    let url = {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
        remote.url().unwrap_or("").to_string()
    };

    let callbacks = auth::make_callbacks(&app, &url).await?;

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
