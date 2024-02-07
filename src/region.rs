use once_cell::sync::Lazy;

pub static REGIONS: Lazy<Vec<String>> = Lazy::new(|| {
    let content = include_str!("../assets/regions.json");
    let mut regions: Vec<String> = serde_json::from_str(content).unwrap();
    regions.push("香港".to_string());
    regions.push("澳洲".to_string());
    regions
});
