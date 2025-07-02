use color_eyre::Report;
use convertor::cache::{Cache, CacheKey};
use convertor::client::Client;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_cache_file_roundtrip() {
    // 定义一个简单的 CacheKey
    let key = CacheKey {
        prefix: "unit_test".to_string(),
        hash: "mykey".to_string(),
        client: Client::Surge,
    };

    let tmp_dir = tempdir().unwrap();
    let cache = Cache::new(10, tmp_dir.path(), Duration::from_secs(10));

    let val = cache
        .try_get_with(key.clone(), async { Ok::<_, Report>("hello cache".to_string()) })
        .await
        .unwrap();

    assert_eq!(val, "hello cache");

    // 再次获取，应该命中缓存
    let val2 = cache
        .try_get_with(key, async {
            panic!("Should not hit loader");
            #[allow(unreachable_code)]
            Ok::<_, Report>("never reached".to_string())
        })
        .await
        .unwrap();

    assert_eq!(val2, "hello cache");
}
