use crate::core::error::{ParseError, RenderError};

pub type ParseResult<T> = Result<T, ParseError>;
pub type RenderResult<T> = Result<T, RenderError>;
