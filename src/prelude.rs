pub use crate::common::*;
pub use crate::keyed::*;
pub use crate::loading::spinner;
pub use crate::padding::{Padded, Padding};
pub use crate::some::*;
pub use crate::tracked::*;
pub use crate::use_sse::use_sse;
pub use bson::oid::ObjectId;
pub use derive_where::derive_where;
pub use dioxus::logger::tracing;
pub use dioxus::prelude::*;
pub use dioxus_fullstack::FullstackContext;
pub use dioxus_fullstack::JsonStream;
pub use dioxus_router::Link;
pub use dioxus_router::Routable;
pub use dioxus_router::Router;
pub use dioxus_router::navigator;
pub use reqwasm::http::Request;
pub use std::collections::HashMap;
pub use std::collections::HashSet;
pub use std::fmt::Debug;
pub use std::hash::Hash;
pub use std::rc::Rc;
#[cfg(feature = "server")]
mod server_prelude {
    pub use crate::server::auth::*;
    pub use axum::Extension;
    pub use axum_extra::extract::cookie::Cookie;
    pub use axum_extra::extract::cookie::CookieJar;
    pub use tower_cookies::Cookies;
}

#[cfg(feature = "server")]
pub use server_prelude::*;
