use crate::{
    AppState,
    models::{Link, LinkId},
};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

pub async fn get_link_redirect_handler(
    State(state): State<AppState>,
    Path(link_id): Path<LinkId>,
) -> Result<Redirect, StatusCode> {
    let config = state.config.read().await;
    let Some(link) = config.links().get(&link_id) else {
        debug!("Link ID '{}' not found in configuration", link_id);
        return Err(StatusCode::NOT_FOUND);
    };

    if !link_valid(&link_id, link).await.map_err(|err| {
        error!("Failed to validate Link ID '{}': {}", link_id, err);
        StatusCode::INTERNAL_SERVER_ERROR
    })? {
        return Err(StatusCode::NOT_FOUND);
    }

    debug!("Redirecting Link ID '{}' -> '{}'", link_id, link.redirect,);

    Ok(Redirect::temporary(link.redirect.as_str()))
}

async fn link_valid(link_id: &str, link_data: &Link) -> Result<bool> {
    // Ensure the link is still enabled.
    if link_data.disabled {
        debug!("Link ID '{}' is disabled", link_id);
        return Ok(false);
    }

    // Ensure the link has not expired.
    if let Some(invalid_after) = link_data.invalid_after
        && invalid_after < SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
    {
        debug!(
            "Link ID '{}' has expired - invalid after {}",
            link_id, invalid_after
        );
        return Ok(false);
    }

    Ok(true)
}
