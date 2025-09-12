use headers::HeaderMap;
use reqwest::{Method, StatusCode};
use std::fmt::{Debug, Display, Formatter};
use url::Url;

#[derive(Debug)]
pub enum ApiResponse<T>
where
    T: Debug,
{
    Success(T),
    Failed(Box<ApiFailed>),
}

#[derive(Debug)]
pub struct ApiFailed {
    pub url: Url,
    pub method: Method,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: String,
}

impl Display for ApiFailed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}] {}", self.method, self.url)?;
        writeln!(f, "Status: {}", self.status)?;
        writeln!(f, "Headers:")?;
        for (key, value) in self.headers.iter() {
            writeln!(f, "\t{key}: {value:?}")?;
        }
        writeln!(f, "Body:")?;
        write!(f, "{}", self.body)
    }
}
