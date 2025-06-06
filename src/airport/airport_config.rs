pub struct AirportConfig {
    pub base_url: &'static str,
    pub prefix_path: &'static str,
    pub one_password_key: &'static str,
    pub login_api: ConfigApi,
    pub reset_subscription_url: ConfigApi,
    pub get_subscription_url: ConfigApi,
    pub get_subscription_log: ConfigApi,
}

pub struct ConfigApi {
    pub api_path: &'static str,
    pub json_path: &'static str,
}
