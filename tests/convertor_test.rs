mod common;
mod server;
mod cli;

use convertor::common::once::init_backtrace;
use include_dir::{Dir, include_dir};
use std::path::{Path, PathBuf};

static SURGE_MOCK_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/test-assets/surge/convertor");
static CLASH_MOCK_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/test-assets/clash/convertor");

pub fn init_test_base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("test-assets")
}

pub fn init_test() -> PathBuf {
    let base_dir = init_test_base_dir();
    init_backtrace();
    base_dir
}
