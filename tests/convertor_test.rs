mod cli;
mod common;
mod server;

use convertor::common::encrypt::nonce_rng_use_seed;
use convertor::common::once::{init_backtrace, init_log};
use std::path::{Path, PathBuf};

pub fn init_test_base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("test-assets")
}

pub fn init_test() -> PathBuf {
    let base_dir = init_test_base_dir();
    init_backtrace();
    init_log(None);
    nonce_rng_use_seed([0u8; 32]);
    base_dir
}
