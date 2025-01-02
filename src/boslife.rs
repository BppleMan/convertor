use crate::encrypt::encrypt;
use crate::op;
use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Result;
use reqwest::Url;
use std::path::Path;

const BASE_URL: &str = "https://boslife.net/api/v1";
const LOGIN_API: &str = "/passport/auth/login";
const RESET_API: &str = "/user/resetSecurity";
const GET_SUBSCRIPTION_API: &str = "/user/getSubscribe";

async fn login(client: &reqwest::Client) -> Result<String> {
    let login_url = format!("{}{}", BASE_URL, LOGIN_API);
    let boslife_identity = op::get_item("BosLife").await?;
    let response = client
        .post(login_url)
        .header(
            "User-Agent",
            format!("convertor/{}", env!("CARGO_PKG_VERSION")),
        )
        .form(&[
            ("email", &boslife_identity.username),
            ("password", &boslife_identity.password),
        ])
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let auth_token = response
        .as_object()
        .ok_or_else(|| eyre!("response not a json object"))?
        .get("data")
        .ok_or_else(|| eyre!("response object not found [data]"))?
        .as_object()
        .ok_or_else(|| eyre!("response[data] not a json object"))?
        .get("auth_data")
        .ok_or_else(|| eyre!("response[data] object not found [auth_data]"))?
        .as_str()
        .ok_or_else(|| eyre!("response[data][auth_data] not a json object"))?
        .to_string();

    Ok(auth_token)
}

pub async fn update_profile<S: AsRef<str>>(
    surge_subscription_host: S,
    refresh_token: bool,
) -> Result<()> {
    let client = reqwest::Client::new();

    let auth_token = login(&client).await?;

    let subscription_url = if refresh_token {
        let reset_url = format!("{}{}", BASE_URL, RESET_API);

        let response = client
            .get(reset_url)
            .header(
                "User-Agent",
                format!("convertor/{}", env!("CARGO_PKG_VERSION")),
            )
            .header("Authorization", auth_token)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Url::parse(
            response
                .as_object()
                .ok_or_else(|| eyre!("response not a json object"))?
                .get("data")
                .ok_or_else(|| eyre!("response object not found [data]"))?
                .as_str()
                .ok_or_else(|| eyre!("response[data] not a string"))?,
        )?
    } else {
        let get_subscription_url =
            format!("{}{}", BASE_URL, GET_SUBSCRIPTION_API);

        let response = client
            .get(get_subscription_url)
            .header(
                "User-Agent",
                format!("convertor/{}", env!("CARGO_PKG_VERSION")),
            )
            .header("Authorization", auth_token)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Url::parse(
            response
                .as_object()
                .ok_or_else(|| eyre!("response not a json object"))?
                .get("data")
                .ok_or_else(|| eyre!("response object not found [data]"))?
                .as_object()
                .ok_or_else(|| eyre!("response[data] not a json object"))?
                .get("subscribe_url")
                .ok_or_else(|| {
                    eyre!("response[data] object not found [subscribe_url]")
                })?
                .as_str()
                .ok_or_else(|| {
                    eyre!("response[data][subscribe_url] not a string")
                })?,
        )?
    };

    let surge_subscription_url =
        SurgeSubscriptionUrl::new(surge_subscription_host, &subscription_url)?;

    let icloud_env = std::env::var("ICLOUD")?;
    let icloud_path = Path::new(&icloud_env);
    let ns_surge_path = icloud_path
        .parent()
        .ok_or_eyre("not found icloud's parent")?
        .join("iCloud~com~nssurge~inc")
        .join("Documents");

    update_subscription(
        &subscription_url,
        &surge_subscription_url,
        &ns_surge_path,
    )
    .await?;

    update_rule_set(&surge_subscription_url, &ns_surge_path).await?;

    Ok(())
}

async fn update_subscription<P: AsRef<Path>>(
    subscription_url: &Url,
    surge_subscription_url: &SurgeSubscriptionUrl,
    ns_surge_path: P,
) -> Result<()> {
    // update BosLife.conf subscription
    let boslife_conf = format!(
        "#!MANAGED-CONFIG {subscription_url} interval=259200 strict=true"
    );
    println!("BosLife.conf: {}", boslife_conf);
    update_conf(&ns_surge_path, "BosLife", &boslife_conf).await?;

    // update surge.conf subscription
    let surge_conf = format!(
        r#"#!MANAGED-CONFIG {} interval=259200 strict=true"#,
        surge_subscription_url.create_surge_api()?
    );
    println!("surge.conf: {}", surge_conf);
    update_conf(&ns_surge_path, "surge", &surge_conf).await?;
    Ok(())
}

async fn update_conf<P: AsRef<Path>, N: AsRef<str>, S: AsRef<str>>(
    ns_surge_path: P,
    name: N,
    sub_url: S,
) -> Result<()> {
    let path = ns_surge_path
        .as_ref()
        .join("surge")
        .join(format!("{}.conf", name.as_ref()));
    let mut content = tokio::fs::read_to_string(&path).await?;
    let mut lines = content.lines().collect::<Vec<_>>();
    lines[0] = sub_url.as_ref();
    content = lines.join("\n");
    tokio::fs::write(path, &content).await?;
    Ok(())
}

async fn update_rule_set<P: AsRef<Path>>(
    surge_subscription_url: &SurgeSubscriptionUrl,
    ns_surge_path: P,
) -> Result<()> {
    update_dconf(
        surge_subscription_url,
        &ns_surge_path,
        RuleSetType::BosLifeSubscription,
    )
    .await?;
    update_dconf(
        surge_subscription_url,
        &ns_surge_path,
        RuleSetType::BosLifePolicy,
    )
    .await?;
    update_dconf(
        surge_subscription_url,
        &ns_surge_path,
        RuleSetType::BosLifeNoResolvePolicy,
    )
    .await?;
    update_dconf(
        surge_subscription_url,
        &ns_surge_path,
        RuleSetType::BosLifeForceRemoteDnsPolicy,
    )
    .await?;
    update_dconf(
        surge_subscription_url,
        &ns_surge_path,
        RuleSetType::DirectPolicy,
    )
    .await?;
    update_dconf(
        surge_subscription_url,
        &ns_surge_path,
        RuleSetType::DirectNoResolvePolicy,
    )
    .await?;
    update_dconf(
        surge_subscription_url,
        &ns_surge_path,
        RuleSetType::DirectForceRemoteDnsPolicy,
    )
    .await?;
    Ok(())
}

async fn update_dconf<P: AsRef<Path>>(
    surge_subscription_url: &SurgeSubscriptionUrl,
    ns_surge_path: P,
    rule_set_type: RuleSetType,
) -> Result<()> {
    let path = ns_surge_path.as_ref().join("surge").join("rules.dconf");
    let content = tokio::fs::read_to_string(&path).await?;
    let mut lines = content.lines().collect::<Vec<_>>();
    let to_be_replace = lines
        .iter()
        .position(|line| line.contains(rule_set_type.name()))
        .ok_or_else(|| eyre!("rule set {} not found", rule_set_type.name()))?;
    let rule_set_url =
        surge_subscription_url.create_rule_set_api(&rule_set_type)?;
    let rule_set = match rule_set_type {
        RuleSetType::BosLifeSubscription
        | RuleSetType::BosLifePolicy
        | RuleSetType::BosLifeNoResolvePolicy
        | RuleSetType::BosLifeForceRemoteDnsPolicy
        | RuleSetType::DirectPolicy
        | RuleSetType::DirectNoResolvePolicy => format!(
            r#"RULE-SET,"{rule_set_url}",{} {}"#,
            rule_set_type.group(),
            rule_set_type.comment()
        ),
        RuleSetType::DirectForceRemoteDnsPolicy => format!(
            r#"// RULE-SET,"{rule_set_url}",{} {}"#,
            rule_set_type.group(),
            rule_set_type.comment()
        ),
    };
    lines[to_be_replace] = &rule_set;
    let content = lines.join("\n");
    tokio::fs::write(path, &content).await?;
    Ok(())
}

#[derive(Debug, Clone)]
struct SurgeSubscriptionUrl {
    host: String,
    base_url: String,
    token: String,
}

impl SurgeSubscriptionUrl {
    fn new<S: AsRef<str>>(
        surge_subscription_host: S,
        subscription_url: &Url,
    ) -> Result<Self> {
        let base_url = format!(
            "{}{}",
            subscription_url.origin().ascii_serialization(),
            subscription_url.path()
        );
        let token = subscription_url
            .query_pairs()
            .find(|(k, _)| k == "token")
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| eyre!("token not found"))?;
        let secret = std::env::var("CONVERTOR_SECRET")?;
        let encrypted_token = encrypt(secret.as_ref(), &token)?;
        let encoded_base_url = urlencoding::encode(&base_url).to_string();
        let encoded_encrypted_token =
            urlencoding::encode(&encrypted_token).to_string();
        Ok(Self {
            host: surge_subscription_host.as_ref().to_string(),
            base_url: encoded_base_url,
            token: encoded_encrypted_token,
        })
    }

    fn create_surge_api(&self) -> Result<Url> {
        let mut url = Url::parse(&self.host)?.join("surge")?;
        url.query_pairs_mut()
            .append_pair("base_url", &self.base_url)
            .append_pair("token", &self.token);
        Ok(url)
    }

    fn create_rule_set_api(&self, rule_set_type: &RuleSetType) -> Result<Url> {
        let mut url = Url::parse(&self.host)?.join("surge/rule_set")?;
        url.query_pairs_mut()
            .append_pair("base_url", &self.base_url)
            .append_pair("token", &self.token);
        match rule_set_type {
            RuleSetType::BosLifeSubscription => {
                url.query_pairs_mut().append_pair("boslife", "true")
            }
            RuleSetType::BosLifePolicy => {
                url.query_pairs_mut().append_pair("policies", "BosLife")
            }
            RuleSetType::BosLifeNoResolvePolicy => url
                .query_pairs_mut()
                .append_pair("policies", "BosLife|no-resolve"),
            RuleSetType::BosLifeForceRemoteDnsPolicy => url
                .query_pairs_mut()
                .append_pair("policies", "BosLife|force-remote-dns"),
            RuleSetType::DirectPolicy => {
                url.query_pairs_mut().append_pair("policies", "DIRECT")
            }
            RuleSetType::DirectNoResolvePolicy => url
                .query_pairs_mut()
                .append_pair("policies", "DIRECT|no-resolve"),
            RuleSetType::DirectForceRemoteDnsPolicy => url
                .query_pairs_mut()
                .append_pair("policies", "DIRECT|force-remote-dns"),
        };
        Ok(url)
    }
}

enum RuleSetType {
    BosLifeSubscription,
    BosLifePolicy,
    BosLifeNoResolvePolicy,
    BosLifeForceRemoteDnsPolicy,
    DirectPolicy,
    DirectNoResolvePolicy,
    DirectForceRemoteDnsPolicy,
}

impl RuleSetType {
    fn name(&self) -> &'static str {
        match self {
            RuleSetType::BosLifeSubscription => "[BosLife Subscription]",
            RuleSetType::BosLifePolicy => "[BosLife Policy]",
            RuleSetType::BosLifeNoResolvePolicy => {
                "[BosLife No Resolve Policy]"
            }
            RuleSetType::BosLifeForceRemoteDnsPolicy => {
                "[BosLife Force Remote Dns Policy]"
            }
            RuleSetType::DirectPolicy => "[Direct Policy]",
            RuleSetType::DirectNoResolvePolicy => "[Direct No Resolve Policy]",
            RuleSetType::DirectForceRemoteDnsPolicy => {
                "[Direct Force Remote Dns Policy]"
            }
        }
    }

    fn comment(&self) -> String {
        format!(
            r#"// Added for {} by convertor/{}"#,
            self.name(),
            env!("CARGO_PKG_VERSION")
        )
    }

    fn group(&self) -> &'static str {
        match self {
            RuleSetType::BosLifeSubscription => "DIRECT",
            RuleSetType::BosLifePolicy => "BosLife",
            RuleSetType::BosLifeNoResolvePolicy => "BosLife,no-resolve",
            RuleSetType::BosLifeForceRemoteDnsPolicy => {
                "BosLife,force-remote-dns"
            }
            RuleSetType::DirectPolicy => "DIRECT",
            RuleSetType::DirectNoResolvePolicy => "DIRECT,no-resolve",
            RuleSetType::DirectForceRemoteDnsPolicy => {
                "DIRECT,force-remote-dns"
            }
        }
    }
}
