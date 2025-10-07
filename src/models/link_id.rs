use anyhow::{Result, bail};
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LinkId(String);

impl LinkId {
    pub fn new<S: Into<String>>(id: S) -> Result<Self> {
        const RESERVED_IDS: [&str; 1] = ["api"];
        const FORBIDDEN_CHARS: [char; 1] = ['\\'];
        let id = id.into();

        // Validation: links cannot be empty
        if id.is_empty() {
            bail!("link ID is empty");
        }

        for chr in id.chars() {
            // Validation: links cannot contain whitespace
            if chr.is_whitespace() {
                bail!("link ID contains whitespace");
            }
            // Validation: links cannot contain forbidden characters
            if FORBIDDEN_CHARS.contains(&chr) {
                bail!("link ID contains one (or more) forbidden characters: {FORBIDDEN_CHARS:?}");
            }
        }

        // Validation: links cannot use reserved IDs
        if RESERVED_IDS.contains(&id.as_str()) {
            bail!("link ID reserved for internal use: '{id}'");
        }

        // Validation: links cannot start or end with a /
        if id.starts_with('/') || id.ends_with('/') {
            bail!("link ID starts or ends with a '/'")
        }

        // TODO: Remove when confirmed stable with no issues.
        if id.contains('/') {
            warn!(
                "link ID '{id}': IDs containing a '/' are an experimental feature and may break in future updates"
            );
        }

        Ok(LinkId(id))
    }
}

impl<'de> Deserialize<'de> for LinkId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        LinkId::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_validation() {
        // Valid cases
        assert!(LinkId::new("valid-id").is_ok());
        assert!(LinkId::new("with_underscore").is_ok());
        assert!(LinkId::new("with/slash").is_ok()); // experimental

        // Invalid cases
        assert!(LinkId::new("").is_err()); // empty
        assert!(LinkId::new("has space").is_err()); // whitespace
        assert!(LinkId::new("has\\backslash").is_err()); // forbidden char
        assert!(LinkId::new("api").is_err()); // reserved
        assert!(LinkId::new("/leading").is_err()); // leading slash
        assert!(LinkId::new("trailing/").is_err()); // trailing slash
    }
}
