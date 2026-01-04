use crate::{
    AppState,
    models::{Link, LinkId},
};
use anyhow::Result;
use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::Redirect,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

pub async fn get_link_redirect_handler(
    State(state): State<AppState>,
    link_id: LinkId,
    uri: Uri,
) -> Result<Redirect, (StatusCode, &'static str)> {
    let config = state.config.read().await;
    let redirect_url = match config.links().get(&link_id) {
        Some(link) if link_valid(&link_id, link).await => link
            .make_redirect_for_path(&link_id, uri.path())
            .map_err(|_err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred.",
                )
            })?,
        _ => {
            return Err((
                StatusCode::NOT_FOUND,
                "This link does not exist or is no longer available.",
            ));
        }
    };
    debug!("Redirecting Link ID '{}' -> '{}'", link_id, redirect_url);
    Ok(Redirect::temporary(redirect_url.as_str()))
}

async fn link_valid(link_id: &str, link_data: &Link) -> bool {
    if link_data.disabled {
        debug!("Link ID '{}' is disabled", link_id);
        return false;
    }

    if let Some(invalid_after) = link_data.invalid_after
        && invalid_after
            < SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_secs()
    {
        debug!(
            "Link ID '{}' has expired - invalid after {}",
            link_id, invalid_after
        );
        return false;
    }

    true
}
