use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Link {
    /// The URL including scheme to redirect to upon visiting.
    pub redirect: Url,
    #[serde(default)]
    /// Whether the link is currently disabled and should be treated like it does not exist.
    pub disabled: bool,
    /// A UNIX-Timestamp in seconds specifying when the link will be invalid.
    pub invalid_after: Option<u64>,
}
