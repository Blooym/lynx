use crate::models::LinkId;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::borrow::Cow;
use url::Url;

#[derive(Deserialize)]
pub struct Link {
    /// The URL (including scheme) to redirect to when this link is visited.
    pub redirect: Url,

    #[serde(default)]
    /// Whether the link is currently disabled and should be treated as non-existent.
    pub disabled: bool,

    /// A Unix timestamp (in seconds) after which the link becomes invalid.
    pub invalid_after: Option<u64>,

    #[serde(default)]
    /// Controls how path components after the short link ID are treated.
    pub append_mode: UrlAppendMode,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub enum UrlAppendMode {
    /// Don't append any components to the redirect URL.
    #[default]
    None,

    /// Append only the path component to the redirect URL.
    Path,

    /// Append only the path component to the redirect URL. Also retain query parameters from the original url.
    PathPreserveQuery,
}

impl Link {
    pub fn make_redirect_for_path(&self, link_id: &LinkId, path: &str) -> Result<Cow<'_, Url>> {
        match self.append_mode {
            UrlAppendMode::Path | UrlAppendMode::PathPreserveQuery => {
                match path.strip_prefix(&format!("/{}/", link_id)) {
                    Some(trailing) if !trailing.is_empty() => {
                        let mut base = self.redirect.clone();
                        if !base.path().ends_with('/') {
                            base.set_path(&format!("{}/", base.path()));
                        }
                        let url = {
                            let mut url = base.join(trailing).with_context(|| {
                                format!("failed to parse a new url for {link_id} {trailing}")
                            })?;
                            if matches!(self.append_mode, UrlAppendMode::PathPreserveQuery) {
                                url.set_query(base.query());
                            }
                            url
                        };
                        Ok(Cow::Owned(url))
                    }
                    _ => Ok(Cow::Borrowed(&self.redirect)),
                }
            }
            UrlAppendMode::None => Ok(Cow::Borrowed(&self.redirect)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_link(url: Url, append_mode: UrlAppendMode) -> Link {
        Link {
            append_mode,
            disabled: false,
            invalid_after: None,
            redirect: url,
        }
    }

    // UrlAppendMode::None tests
    #[test]
    fn test_none_with_no_trailing() {
        let link = make_link(
            Url::parse("https://example.com/base").unwrap(),
            UrlAppendMode::None,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link.make_redirect_for_path(&link_id, "/short").unwrap();
        assert_eq!(result.as_str(), "https://example.com/base");
    }

    #[test]
    fn test_none_with_trailing_path() {
        let link = make_link(
            Url::parse("https://example.com/base").unwrap(),
            UrlAppendMode::None,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link
            .make_redirect_for_path(&link_id, "/short/extra/path")
            .unwrap();
        assert_eq!(result.as_str(), "https://example.com/base");
    }

    // UrlAppendMode::Path tests
    #[test]
    fn test_path_with_no_trailing() {
        let link = make_link(
            Url::parse("https://example.com/base").unwrap(),
            UrlAppendMode::Path,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link.make_redirect_for_path(&link_id, "/short").unwrap();
        assert_eq!(result.as_str(), "https://example.com/base");
    }

    #[test]
    fn test_path_with_trailing_path() {
        let link = make_link(
            Url::parse("https://example.com/base").unwrap(),
            UrlAppendMode::Path,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link
            .make_redirect_for_path(&link_id, "/short/extra/path")
            .unwrap();
        assert_eq!(result.as_str(), "https://example.com/base/extra/path");
    }

    #[test]
    fn test_path_does_not_preserve_query_params() {
        let link = make_link(
            Url::parse("https://example.com/base?existing=param").unwrap(),
            UrlAppendMode::Path,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link
            .make_redirect_for_path(&link_id, "/short/extra/path")
            .unwrap();
        assert_eq!(result.as_str(), "https://example.com/base/extra/path");
    }

    // UrlAppendMode::PathPreserveQuery tests
    #[test]
    fn test_path_preserve_query_with_no_trailing() {
        let link = make_link(
            Url::parse("https://example.com/base").unwrap(),
            UrlAppendMode::PathPreserveQuery,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link.make_redirect_for_path(&link_id, "/short").unwrap();
        assert_eq!(result.as_str(), "https://example.com/base");
    }

    #[test]
    fn test_path_preserve_query_with_trailing_path_no_query() {
        let link = make_link(
            Url::parse("https://example.com/base").unwrap(),
            UrlAppendMode::PathPreserveQuery,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link
            .make_redirect_for_path(&link_id, "/short/extra/path")
            .unwrap();
        assert_eq!(result.as_str(), "https://example.com/base/extra/path");
    }

    #[test]
    fn test_path_preserve_query_with_trailing_path_and_existing_query() {
        let link = make_link(
            Url::parse("https://example.com/base?existing=param&foo=bar").unwrap(),
            UrlAppendMode::PathPreserveQuery,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link
            .make_redirect_for_path(&link_id, "/short/extra/path")
            .unwrap();
        assert_eq!(
            result.as_str(),
            "https://example.com/base/extra/path?existing=param&foo=bar"
        );
    }

    #[test]
    fn test_path_preserve_query_preserves_only_base_query() {
        let link = make_link(
            Url::parse("https://example.com/base?keep=this").unwrap(),
            UrlAppendMode::PathPreserveQuery,
        );
        let link_id = LinkId::new("short").unwrap();

        let result = link
            .make_redirect_for_path(&link_id, "/short/path")
            .unwrap();
        assert_eq!(result.as_str(), "https://example.com/base/path?keep=this");
    }
}
