use crate::profile::error::ParseError;

pub mod surge_parser;

pub type Result<T> = core::result::Result<T, ParseError>;
