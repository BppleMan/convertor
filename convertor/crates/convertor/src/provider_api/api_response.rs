use headers::HeaderMap;
use reqwest::{Method, StatusCode};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use url::Url;

#[derive(Debug)]
pub enum ApiResponse<T>
where
    T: Debug,
{
    Success(T),
    Failed(Box<ApiFailed>),
}

#[derive(Debug, Error)]
pub struct ApiFailed {
    pub request_url: Url,
    pub request_method: Method,
    pub request_headers: HeaderMap,
    pub response_status: StatusCode,
    pub response_headers: HeaderMap,
    pub response_body: String,
}

impl Display for ApiFailed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}] {}", self.request_method, self.request_url)?;
        writeln!(f, "Request Headers:")?;
        for (key, value) in self.request_headers.iter() {
            writeln!(f, "\t{key}: {value:?}")?;
        }
        writeln!(f, "Response Status: {}", self.response_status)?;
        writeln!(f, "Response Headers:")?;
        for (key, value) in self.response_headers.iter() {
            writeln!(f, "\t{key}: {value:?}")?;
        }
        writeln!(f, "Response Body:")?;
        write!(f, "{}", self.response_body)
    }
}
