use crate::args::{Arch, Profile, Registry};

pub struct Tag {
    pub user: String,
    pub name: String,
    pub version: String,
    pub project: String,
    pub profile: Profile,
}

impl Tag {
    pub fn new(user: impl AsRef<str>, project: impl AsRef<str>, profile: Profile) -> Self {
        let user = user.as_ref().to_string();
        let project = project.as_ref().to_string();
        let name = "convd".to_string();
        let version = env!("CARGO_PKG_VERSION").to_string();
        Self {
            user,
            name,
            version,
            project,
            profile,
        }
    }

    pub fn local(&self, arch: Option<Arch>) -> String {
        format!(
            "local/{}/{}/{}:{}{}{}",
            self.user,
            self.project,
            self.name,
            self.version,
            self.profile.as_image_profile(),
            arch.map(|a| format!("-{}", a.as_image_tag())).unwrap_or_default(),
        )
    }

    pub fn remote(&self, registry: &Registry, arch: Option<Arch>) -> String {
        format!(
            "{}/{}/{}/{}:{}{}{}",
            registry.as_url(),
            self.user,
            self.project,
            self.name,
            self.version,
            self.profile.as_image_profile(),
            arch.map(|a| format!("-{}", a.as_image_tag())).unwrap_or_default(),
        )
    }
}
