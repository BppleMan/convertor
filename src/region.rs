use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static REGIONS: Lazy<Vec<Region>> = Lazy::new(|| {
    let content = include_str!("../assets/regions.json");
    serde_json::from_str(content).unwrap()
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Region {
    pub code: String,
    pub en: String,
    pub cn: String,
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
