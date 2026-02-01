use crate::args::{Arch, Profile, Registry, Version};

pub struct Tag {
    pub user: String,
    pub name: String,
    pub version: Version,
    pub project: String,
    pub profile: Profile,
}

impl Tag {
    pub fn new(user: impl AsRef<str>, project: impl AsRef<str>, version: Version, profile: Profile) -> Self {
        let user = user.as_ref().to_string();
        let project = project.as_ref().to_string();
        let name = "convd".to_string();
        Self {
            user,
            name,
            version,
            project,
            profile,
        }
    }

    pub fn local(&self, arch: Option<Arch>, version: Option<&Version>) -> String {
        self.remote(&Registry::Local, arch, version)
    }

    pub fn remote(&self, registry: &Registry, arch: Option<Arch>, version: Option<&Version>) -> String {
        format!(
            "{}/{}/{}/{}:{}{}{}",
            registry.as_url(),
            self.user,
            self.project,
            self.name,
            version.unwrap_or(&self.version),
            self.profile.as_image_profile(),
            arch.map(|a| format!("-{}", a.as_image_tag())).unwrap_or_default(),
        )
    }
}
