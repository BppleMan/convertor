use std::path::PathBuf;
use std::sync::Once;

pub use reqwest::{Method, StatusCode};

pub mod encrypt;
pub mod client;
pub mod cache;
pub mod core;
pub mod config;
pub mod url;
pub mod router;
pub mod api;

pub fn init_base_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().unwrap().join(".convertor");
    #[cfg(not(debug_assertions))]
    let base_dir =
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())).join(".convertor");
    base_dir
}

static INITIALIZED_BACKTRACE: Once = Once::new();

pub fn init_backtrace() {
    INITIALIZED_BACKTRACE.call_once(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
}
