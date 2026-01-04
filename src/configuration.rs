use crate::models::{Link, LinkId};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Deserialize)]
pub struct LynxConfiguration {
    links: HashMap<LinkId, Link>,
}

impl LynxConfiguration {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            bail!("no configuration file exists at {path:?}");
        }
        toml::from_str(&std::fs::read_to_string(path).context("failed to read file")?)
            .context("configuration could not be parsed")
    }

    pub fn links(&self) -> &HashMap<LinkId, Link> {
        &self.links
    }
}
