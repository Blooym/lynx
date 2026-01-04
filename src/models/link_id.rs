use anyhow::{Result, bail};
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use serde::Deserialize;

#[derive(PartialEq, Eq, Hash)]
pub struct LinkId(Box<str>);

impl LinkId {
    pub fn new(id: &str) -> Result<Self> {
        const RESERVED_IDS: [&str; 1] = ["api"];
        const FORBIDDEN_CHARS: [char; 2] = ['\\', '/'];
        let id = id.trim();

        // Validation: links cannot be empty
        if id.is_empty() {
            bail!("link ID is empty");
        }

        // Validation: links cannot use reserved IDs
        if RESERVED_IDS.contains(&id) {
            bail!("link ID reserved for internal use: '{id}'");
        }

        for chr in id.chars() {
            // Validation: links cannot contain whitespace
            if chr.is_whitespace() {
                bail!("link ID contains whitespace");
            }

            // Validation: links cannot contain forbidden characters
            if FORBIDDEN_CHARS.contains(&chr) {
                bail!("link ID contains one or more forbidden characters: {FORBIDDEN_CHARS:?}");
            }
        }

        Ok(LinkId(id.into()))
    }
}

impl<'de> Deserialize<'de> for LinkId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        LinkId::new(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl core::fmt::Display for LinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::ops::Deref for LinkId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for LinkId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let path = parts.uri.path();
        let first_segment = path.trim_start_matches('/').split('/').next().unwrap_or("");
        LinkId::new(first_segment).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong while validating the given LinkId",
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_id() {
        assert!(LinkId::new("validid").is_ok());
        assert!(LinkId::new("valid-id").is_ok());
        assert!(LinkId::new("valid_id").is_ok());
    }

    #[test]
    fn test_empty_id_rejected() {
        assert!(LinkId::new("").is_err());
    }

    #[test]
    fn test_whitespace_rejected() {
        assert!(LinkId::new("has space").is_err());
    }

    #[test]
    fn test_invalid_char_rejected() {
        assert!(LinkId::new("has\\invalidchar").is_err());
        assert!(LinkId::new("with/slash").is_err());
    }

    #[test]
    fn test_reserved_id_rejected() {
        assert!(LinkId::new("api").is_err());
    }
}
