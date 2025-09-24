use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LinkId(String);

impl LinkId {
    pub fn new<S: Into<String>>(id: S) -> Result<Self> {
        const RESERVED_IDS: [&str; 1] = ["api"]; // might want these in the future for exposing server ops
        let id = id.into();
        if id.is_empty() {
            anyhow::bail!("link IDs cannot be empty");
        }
        if id.contains('/') {
            anyhow::bail!("link IDs cannot contain '/' characters");
        }
        if RESERVED_IDS.contains(&id.as_str()) {
            anyhow::bail!("'{}' is a reserved link ID", id);
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
