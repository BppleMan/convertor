use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const REGIONS_CONTENT: &str = include_str!("../assets/regions.json");

static REGIONS: Lazy<Vec<Region>> = Lazy::new(|| serde_json::from_str(REGIONS_CONTENT).unwrap());

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Region {
    pub code: String,
    pub en: String,
    pub cn: String,
    pub icon: String,
}

impl Region {
    pub fn detect(pattern: impl AsRef<str>) -> Option<&'static Self> {
        let pattern = pattern.as_ref();
        REGIONS.iter().find(|r| {
            let variants = [
                r.code.to_string(),
                r.code.to_lowercase(),
                r.en.to_lowercase(),
                r.en.to_uppercase(),
                r.en.replace(' ', "-"),
                r.en.replace(' ', "_"),
                r.en.replace(' ', ""),
                r.cn.to_string(),
            ];
            variants.iter().any(|v| pattern.contains(v))
        })
    }
}
