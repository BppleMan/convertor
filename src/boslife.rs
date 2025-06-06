use crate::convertor_url::ConvertorUrl;
use crate::service::boslife_service::BosLifeService;
use crate::service::service_api::AirportApi;
use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Result;
use std::path::Path;

pub async fn update_profile<S: AsRef<str>>(
    server_addr: S,
    refresh_token: bool,
) -> Result<()> {
    let client = reqwest::Client::new();

    let service = BosLifeService::new(client);
    let auth_token = service.login().await?;

    let subscription_url = if refresh_token {
        service.reset_subscription_url(&auth_token).await?
    } else {
        service.get_subscription_url(&auth_token).await?
    };

    let convertor_url = ConvertorUrl::new(server_addr, &subscription_url)?;

    let icloud_env = std::env::var("ICLOUD")?;
    let icloud_path = Path::new(&icloud_env);
    let ns_surge_path = icloud_path
        .parent()
        .ok_or_eyre("not found icloud's parent")?
        .join("iCloud~com~nssurge~inc")
        .join("Documents");

    update_surge_conf(&convertor_url, &ns_surge_path).await?;

    update_rule_set(&convertor_url, &ns_surge_path).await?;

    Ok(())
}

async fn update_surge_conf(
    convertor_url: &ConvertorUrl,
    ns_surge_path: impl AsRef<Path>,
) -> Result<()> {
    // update BosLife.conf subscription
    let boslife_conf = format!(
        "#!MANAGED-CONFIG {} interval=259200 strict=true",
        convertor_url.build_service_url("surge")?
    );
    println!("BosLife.conf: {}", boslife_conf);
    update_conf(&ns_surge_path, "BosLife", &boslife_conf).await?;

    // update surge.conf subscription
    let surge_conf = format!(
        r#"#!MANAGED-CONFIG {} interval=259200 strict=true"#,
        convertor_url.encode_to_convertor_url()?
    );
    println!("surge.conf: {}", surge_conf);
    update_conf(&ns_surge_path, "surge", &surge_conf).await?;
    Ok(())
}

async fn update_conf(
    ns_surge_path: impl AsRef<Path>,
    name: impl AsRef<str>,
    sub_url: impl AsRef<str>,
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

async fn update_rule_set(
    convertor_url: &ConvertorUrl,
    ns_surge_path: impl AsRef<Path>,
) -> Result<()> {
    update_dconf(
        convertor_url,
        &ns_surge_path,
        RuleSetType::BosLifeSubscription,
    )
    .await?;
    update_dconf(convertor_url, &ns_surge_path, RuleSetType::BosLifePolicy)
        .await?;
    update_dconf(
        convertor_url,
        &ns_surge_path,
        RuleSetType::BosLifeNoResolvePolicy,
    )
    .await?;
    update_dconf(
        convertor_url,
        &ns_surge_path,
        RuleSetType::BosLifeForceRemoteDnsPolicy,
    )
    .await?;
    update_dconf(convertor_url, &ns_surge_path, RuleSetType::DirectPolicy)
        .await?;
    update_dconf(
        convertor_url,
        &ns_surge_path,
        RuleSetType::DirectNoResolvePolicy,
    )
    .await?;
    update_dconf(
        convertor_url,
        &ns_surge_path,
        RuleSetType::DirectForceRemoteDnsPolicy,
    )
    .await?;
    Ok(())
}

async fn update_dconf<P: AsRef<Path>>(
    surge_subscription_url: &ConvertorUrl,
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

pub enum RuleSetType {
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
