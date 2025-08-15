use crate::common::config::proxy_client::ProxyClient;
use color_eyre::Report;
use moka::future::Cache as MokaCache;
use redis::AsyncTypedCommands;
use redis::aio::ConnectionManager;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};

pub const CACHED_AUTH_TOKEN_KEY: &str = "cached:auth_token";
pub const CACHED_PROFILE_KEY: &str = "cached:profile";
pub const CACHED_UNI_SUB_URL_KEY: &str = "cached:uni_sub_url";
pub const CACHED_SUB_LOGS_KEY: &str = "cached:sub_logs";

#[derive(Clone)]
pub struct Cache<K, V>
where
    K: Hash + Eq + Clone + Debug + Display + Send + Sync + 'static,
    V: Clone + From<String> + ToString + Send + Sync + 'static,
{
    memory: MokaCache<CacheKey<K>, V>,
    redis: ConnectionManager,
    time_to_live: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone + Debug + Display + Send + Sync + 'static,
    V: Clone + From<String> + ToString + Send + Sync + 'static,
{
    pub fn new(redis: ConnectionManager, capacity: u64, time_to_live: Duration) -> Self {
        let memory = moka::future::Cache::builder()
            .max_capacity(capacity)
            .time_to_live(time_to_live)
            .build();
        Self {
            memory,
            redis,
            time_to_live,
        }
    }

    pub async fn try_get_with<F>(&self, key: CacheKey<K>, init: F) -> Result<V, Arc<Report>>
    where
        F: Future<Output = color_eyre::Result<V>>,
    {
        self.memory
            .try_get_with(key.clone(), async { self.try_get_from_redis(key, init).await })
            .await
    }

    async fn try_get_from_redis<F>(&self, key: CacheKey<K>, init: F) -> Result<V, Report>
    where
        F: Future<Output = color_eyre::Result<V>>,
    {
        let mut redis = self.redis.clone();
        let redis_key = key.as_redis_key();
        if let Ok(Some(raw)) = redis.get(&redis_key).await {
            debug!("命中 Redis 缓存: {}", redis_key);
            return Ok(V::from(raw));
        }

        // 如果 Redis 中没有命中，则尝试从文件系统获取
        let value = init.await?;
        let raw = value.to_string();

        // 将结果存入 Redis
        if let Err(e) = redis.set_ex(redis_key, raw, self.time_to_live.as_secs()).await {
            error!("无法将缓存写入 Redis: {}", e);
        }

        Ok(value)
    }

    // async fn try_get_from_file_with<F>(&self, key: &CacheKey<K>, init: F) -> Result<V, Report>
    // where
    //     F: Future<Output = color_eyre::Result<V>>,
    // {
    //     let now = Self::now_ts();
    //
    //     if let Some(path) = self.find_valid_cache_file(key, now).await? {
    //         debug!("命中缓存文件: {}", path.display());
    //         let raw = tokio::fs::read_to_string(path).await?;
    //         return Ok(V::from(raw));
    //     }
    //
    //     let value = init.await?;
    //     let raw = value.to_string();
    //
    //     let expire_time = Local::now() + chrono::Duration::from_std(self.time_to_live)?;
    //     let full_path = key.to_full_path(&self.cache_dir, expire_time);
    //
    //     // 确保目录存在
    //     if let Some(parent) = full_path.parent() {
    //         tokio::fs::create_dir_all(parent).await?;
    //     }
    //
    //     tokio::fs::write(&full_path, raw).await?;
    //     Ok(value)
    // }
    //
    // async fn find_valid_cache_file(&self, key: &CacheKey<K>, now_ts: u64) -> Result<Option<PathBuf>, Report> {
    //     use tokio_stream::{StreamExt, wrappers::ReadDirStream};
    //
    //     let target_dir = self.cache_dir.join(key.prefix_path());
    //     let hash_prefix = key.hash_code();
    //
    //     let mut read_dir = match tokio::fs::read_dir(&target_dir).await {
    //         Ok(rd) => ReadDirStream::new(rd),
    //         Err(_) => return Ok(None), // 无目录直接返回 None
    //     };
    //
    //     while let Some(entry) = read_dir.next().await {
    //         let entry = entry?;
    //         let path = entry.path();
    //
    //         let Some(file_stem) = path.file_stem().and_then(|f| f.to_str()) else {
    //             continue; // 无法读取文件名，跳过
    //         };
    //
    //         let Some((hash, expires_at)) = Self::decode_file_stem(file_stem) else {
    //             continue; // 非法文件名，跳过
    //         };
    //
    //         if hash != hash_prefix {
    //             continue; // 不匹配当前 key，跳过
    //         }
    //
    //         if expires_at > now_ts {
    //             return Ok(Some(path)); // 命中有效缓存
    //         }
    //
    //         // 过期：尝试删除
    //         match tokio::fs::remove_file(&path).await {
    //             Ok(_) => debug!("清理过期缓存文件: {}", path.display()),
    //             Err(e) => error!("无法删除过期缓存文件 {}: {}", path.display(), e),
    //         }
    //     }
    //
    //     Ok(None)
    // }
    //
    // /// 从干净的 file_stem 中提取 `(hash, expires_ts)`
    // /// 要求传入格式为：`<hash_prefix>__<datetime>`
    // /// 如：`abc123__2025-07-02T12-00-00`
    // fn decode_file_stem(file_stem: &str) -> Option<(u64, u64)> {
    //     let (hash, time_str) = file_stem.rsplit_once("__")?;
    //     let naive = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H-%M-%S").ok()?;
    //     let local = Local.from_local_datetime(&naive).single()?;
    //     let hash_code = hash.parse::<u64>().ok()?;
    //     Some((hash_code, local.timestamp() as u64))
    // }
    //
    // fn now_ts() -> u64 {
    //     SystemTime::now()
    //         .duration_since(UNIX_EPOCH)
    //         .expect("System time before epoch")
    //         .as_secs()
    // }
    //
    // fn clone_shallow(&self) -> Self {
    //     Self {
    //         memory: self.memory.clone(),
    //         redis: self.redis.clone(),
    //         capacity: self.capacity,
    //         time_to_live: self.time_to_live,
    //     }
    // }
}

pub trait AsRedisKey {
    fn as_redis_key(&self) -> String;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CacheKey<H: Hash + Eq + Clone + Display + Send + Sync + 'static> {
    pub prefix: String,
    pub hash: H,
    pub client: Option<ProxyClient>,
}

impl<H> AsRedisKey for CacheKey<H>
where
    H: Hash + Eq + Clone + Display + Send + Sync + 'static,
{
    fn as_redis_key(&self) -> String {
        use std::fmt::Write;
        let mut key = format!("convertor:{}", self.prefix);
        if let Some(client) = &self.client {
            write!(&mut key, ":{}", client.as_str()).expect("Failed to write client to key");
        }
        write!(&mut key, ":{}", self.hash).expect("Failed to write hash to key");
        key
    }
}

impl<H> CacheKey<H>
where
    H: Hash + Eq + Clone + Display + Send + Sync + 'static,
{
    pub fn new(prefix: impl AsRef<str>, hash: H, client: Option<ProxyClient>) -> Self {
        Self {
            prefix: prefix.as_ref().to_owned(),
            hash,
            client,
        }
    }

    // /// 生成 hash 文件名前缀，例如 "a4bc1398"
    // pub fn hash_code(&self) -> u64 {
    //     let mut hasher = DefaultHasher::default();
    //     self.hash(&mut hasher);
    //     hasher.finish()
    // }
    //
    // pub fn prefix_path(&self) -> PathBuf {
    //     let mut prefix_path = PathBuf::from(&self.prefix);
    //     if let Some(client) = &self.client {
    //         prefix_path = prefix_path.join(client.as_str())
    //     };
    //     prefix_path
    // }
    //
    // /// 返回相对路径（不含 cache_dir）：<prefix>/<client>/<short_hash>__<expires>.txt
    // pub fn relative_path(&self, expires_at: DateTime<Local>) -> PathBuf {
    //     let file_name = format!("{}__{}.txt", self.hash_code(), expires_at.format("%Y-%m-%dT%H-%M-%S"));
    //     self.prefix_path().join(file_name)
    // }
    //
    // /// 返回完整缓存路径：<cache_dir>/<prefix>/<client>/<hash>__<expires>.txt
    // pub fn to_full_path(&self, cache_dir: impl AsRef<Path>, expires_at: DateTime<Local>) -> PathBuf {
    //     cache_dir.as_ref().join(self.relative_path(expires_at))
    // }
}

impl<H> Display for CacheKey<H>
where
    H: Hash + Eq + Clone + Display + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.prefix)?;
        if let Some(client) = &self.client {
            write!(f, ":{}", client.as_str())?;
        }
        write!(f, ":{}", self.hash)
    }
}
