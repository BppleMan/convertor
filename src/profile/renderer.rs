#![deny(unused, unused_variables)]

use crate::profile::error::RenderError;

pub mod surge_renderer;
pub mod clash_renderer;

pub type Result<T> = core::result::Result<T, RenderError>;
