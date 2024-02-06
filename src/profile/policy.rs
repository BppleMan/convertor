use crate::profile::{Proxy, ProxyGroup};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Policy {
    Proxy(Proxy),
    ProxyGroup(ProxyGroup),
}
