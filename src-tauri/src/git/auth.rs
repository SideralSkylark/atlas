use crate::credentials;
use git2::{CertificateCheckStatus, Cred, RemoteCallbacks};
use tauri::{AppHandle, Runtime};

fn get_domain_from_url(url: &str) -> Option<String> {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?
        .split('/')
        .next()
        .map(|s| s.to_string())
}

pub async fn make_callbacks<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
) -> Result<RemoteCallbacks<'static>, String> {
    let creds = credentials::load_credentials(app).await?;

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
