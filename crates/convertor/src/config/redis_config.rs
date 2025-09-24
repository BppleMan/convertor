use redis::{ConnectionAddr, ConnectionInfo, IntoConnectionInfo, ProtocolVersion, RedisConnectionInfo, RedisResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default)]
    pub db: Option<u32>,
    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
}

impl RedisConfig {
    pub fn build_redis_client(&self) -> Result<redis::Client, RedisConfigError> {
        let config = self.validate()?;
        let redis_client = match config.tls.clone() {
            None => redis::Client::open(config),
            Some(tls) => redis::Client::build_with_tls(config, tls.into()),
        }?;
        Ok(redis_client)
    }

    pub fn validate(&self) -> Result<RedisConfig, RedisConfigError> {
        let RedisConfig {
            mut host,
            port,
            mut username,
            password,
            db,
            prefix,
            mut tls,
        } = self.clone();
        host = host.trim().to_string();
        if host.is_empty() {
            return Err(RedisConfigError::EmptyHost);
        } else if host.contains(':') {
            return Err(RedisConfigError::InvalidHost(host));
        }
        if self.port == 0 {
            return Err(RedisConfigError::ZeroPort);
        }
        username = username.trim().replace("default", "").to_string();

        if let Some(TlsConfig {
            ca_cert,
            client_cert,
            client_key,
        }) = &mut tls
        {
            if let Some(ca_cert) = ca_cert.as_mut() {
                *ca_cert = ca_cert.trim().to_string();
            }
            if let Some(client_cert) = client_cert.as_mut() {
                *client_cert = client_cert.trim().to_string();
            }
            if let Some(client_key) = client_key.as_mut() {
                *client_key = client_key.trim().to_string();
            }
            match (client_cert, client_key) {
                (Some(_), None) => return Err(RedisConfigError::HasClientCertButNoKey),
                (None, Some(_)) => return Err(RedisConfigError::HasClientKeyButNoCert),
                _ => {}
            }
        }

        Ok(RedisConfig {
            host,
            port,
            username,
            password,
            db,
            prefix,
            tls,
        })
    }
}

impl RedisConfig {
    pub fn template() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
            username: "".to_string(),
            password: "yourpassword".to_string(),
            prefix: "convertor:".to_string(),
            db: Some(0),
            tls: Some(TlsConfig::template()),
        }
    }
}

impl TlsConfig {
    pub fn template() -> Self {
        Self {
            ca_cert: Some(
                r#"
-----BEGIN CERTIFICATE-----
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=
-----END CERTIFICATE-----
            "#
                .trim()
                .to_string(),
            ),
            client_cert: Some(
                r#"
-----BEGIN CERTIFICATE-----
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=
-----END CERTIFICATE-----
            "#
                .trim()
                .to_string(),
            ),
            client_key: Some(
                r#"
-----BEGIN PRIVATE KEY-----
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=
-----END PRIVATE KEY-----
            "#
                .trim()
                .to_string(),
            ),
        }
    }
}

impl IntoConnectionInfo for RedisConfig {
    fn into_connection_info(self) -> RedisResult<ConnectionInfo> {
        let connection_info = ConnectionInfo {
            addr: match self.tls {
                None => ConnectionAddr::Tcp(self.host, self.port),
                Some(_tls) => ConnectionAddr::TcpTls {
                    host: self.host,
                    port: self.port,
                    insecure: false,
                    tls_params: None,
                },
            },
            redis: RedisConnectionInfo {
                db: self.db.unwrap_or(0) as i64,
                username: match self.username {
                    ref u if u.is_empty() => None,
                    ref u => Some(u.clone()),
                },
                password: match self.password {
                    ref p if p.is_empty() => None,
                    ref p => Some(p.clone()),
                },
                protocol: ProtocolVersion::RESP3,
            },
        };

        Ok(connection_info)
    }
}

impl From<TlsConfig> for redis::TlsCertificates {
    fn from(value: TlsConfig) -> Self {
        let TlsConfig {
            ca_cert,
            client_cert,
            client_key,
        } = value;
        redis::TlsCertificates {
            client_tls: match (client_cert, client_key) {
                (Some(client_cert), Some(client_key)) => Some(redis::ClientTlsConfig {
                    client_cert: client_cert.into_bytes(),
                    client_key: client_key.into_bytes(),
                }),
                _ => None,
            },
            root_cert: ca_cert.map(|ca_cert| ca_cert.into_bytes()),
        }
    }
}

#[derive(Debug, Error)]
pub enum RedisConfigError {
    #[error("Redis host is empty")]
    EmptyHost,
    #[error("Redis host is invalid: {0}")]
    InvalidHost(String),
    #[error("Redis port is 0")]
    ZeroPort,
    #[error("Redis password is empty")]
    EmptyPassword,
    #[error("TLS configuration error: both client_cert and client_key must be provided together")]
    HasClientCertButNoKey,
    #[error("TLS configuration error: both client_cert and client_key must be provided together")]
    HasClientKeyButNoCert,
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
}

pub fn default_prefix() -> String {
    "convertor:".to_string()
}
