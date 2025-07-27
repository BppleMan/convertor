// use crate::common::config::ConvertorConfig;
// use color_eyre::Result;
// use color_eyre::eyre::eyre;
// use moka::future::Cache;
// use std::sync::Arc;
// use url::Url;
//
// pub struct RawService {
//     pub config: Arc<ConvertorConfig>,
//     pub profile_cache: Cache<Url, String>,
// }
//
// impl RawService {
//     pub fn new(config: Arc<ConvertorConfig>) -> Self {
//         let duration = std::time::Duration::from_secs(60 * 60);
//         let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
//         Self { config, profile_cache }
//     }
//
//     pub async fn raw_profile(&self, url: Url) -> Result<String> {
//         self.profile_cache
//             .try_get_with(url.clone(), async {
//                 // Simulate fetching the raw profile from the URL
//                 // In a real implementation, this would involve making an HTTP request
//                 let raw_profile = format!("Raw profile data for {}", url);
//                 Ok::<String, String>(raw_profile)
//             })
//             .await
//             .map_err(|e| eyre!(e))
//     }
// }
