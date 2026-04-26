// pub use anyhow::{Error, Result, anyhow, bail};
pub use crate::common::*;
pub use anyhow::Result;
pub use anyhow::anyhow;
use axum::response::IntoResponse;
pub use dioxus::core::bail;
use dioxus::server::ServerFnError;
pub use libreauth::pass::HashBuilder;
pub use mongodb::bson::doc;
pub use mongodb::options::{ClientOptions, IndexOptions};
pub use mongodb::{Client, Collection, Database, IndexModel, bson::oid::ObjectId};
pub use serde::{Deserialize, Serialize};

pub enum Error {
    Err(anyhow::Error),
    Handled(ServerFnError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Err(error) => ServerFnError::from(error).into_response(),
            Self::Handled(error) => error.into_response(),
        }
    }
}

impl<E: Into<anyhow::Error>> From<E> for Error {
    fn from(value: E) -> Self {
        Error::Err(value.into())
    }
}

pub fn unauthorized() -> Error {
    Error::Handled(ServerFnError::ServerError {
        message: "unauthorized".into(),
        code: 401,
        details: None,
    })
}
