use std::env;

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub mongo_url: String,
    pub prefix: String,
    pub pem: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            mongo_url: required_env("MONGO")
                .or_else(|_| required_env("MONGO_URL"))
                .context("missing MONGO or MONGO_URL")?,
            prefix: required_env("PREFIX")
                .or_else(|_| required_env("COLLECTION_PREFIX"))
                .context("missing PREFIX or COLLECTION_PREFIX")?,
            pem: required_env("PEM")
                .or_else(|_| required_env("VAPID_PEM"))
                .context("missing PEM or VAPID_PEM")?,
        })
    }
}

fn required_env(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("missing {key}"))
}
