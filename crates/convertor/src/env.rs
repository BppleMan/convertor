use std::fmt::{Display, Formatter};

#[derive(Default, Debug)]
pub enum Env {
    #[default]
    Development,
    Production,
}

impl Env {
    pub const fn current() -> Self {
        #[cfg(debug_assertions)]
        return Env::Development;
        #[cfg(not(debug_assertions))]
        return Env::Production;
    }

    pub fn name(&self) -> &'static str {
        match self {
            Env::Development => "Development",
            Env::Production => "Production",
        }
    }
}

impl Display for Env {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

impl AsRef<str> for Env {
    fn as_ref(&self) -> &str {
        self.name()
    }
}

impl From<Env> for String {
    fn from(value: Env) -> Self {
        value.to_string()
    }
}
