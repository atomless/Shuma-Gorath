use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

#[cfg(test)]
use super::adversary_sim_identity_pool::IdentityPoolEntry;

const USERINFO_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b':')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrustedIngressProxyConfig {
    pub base_proxy_url: String,
    pub auth_token: String,
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn valid_base_proxy_url(base_proxy_url: &str) -> bool {
    let trimmed = base_proxy_url.trim();
    let Some((scheme, remainder)) = trimmed.split_once("://") else {
        return false;
    };
    if !matches!(scheme, "http" | "https") {
        return false;
    }
    if remainder.is_empty()
        || remainder.starts_with('/')
        || remainder.contains('@')
        || remainder.contains('?')
        || remainder.contains('#')
        || remainder.contains('\r')
        || remainder.contains('\n')
    {
        return false;
    }
    true
}

pub(crate) fn trusted_ingress_proxy_config_from_env() -> Option<TrustedIngressProxyConfig> {
    let base_proxy_url = non_empty_env("ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL")?;
    let auth_token = non_empty_env("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN")?;
    if !valid_base_proxy_url(base_proxy_url.as_str())
        || auth_token.contains('\r')
        || auth_token.contains('\n')
    {
        return None;
    }
    Some(TrustedIngressProxyConfig {
        base_proxy_url,
        auth_token,
    })
}

pub(crate) fn trusted_ingress_proxy_url_for_client_ip(
    config: &TrustedIngressProxyConfig,
    client_ip: &str,
) -> Option<String> {
    let normalized_ip = crate::request_validation::parse_ip_addr(client_ip)?;
    if normalized_ip.is_empty() {
        return None;
    }
    let (scheme, remainder) = config.base_proxy_url.split_once("://")?;
    let encoded_ip = utf8_percent_encode(normalized_ip.as_str(), USERINFO_ENCODE_SET).to_string();
    let encoded_token =
        utf8_percent_encode(config.auth_token.as_str(), USERINFO_ENCODE_SET).to_string();
    Some(format!(
        "{scheme}://{encoded_ip}:{encoded_token}@{remainder}"
    ))
}

#[cfg(test)]
pub(crate) fn fixed_proxy_entry_for_client_ip(
    config: &TrustedIngressProxyConfig,
    client_ip: &str,
    label: &str,
    identity_class: &str,
    country_code: &str,
) -> Option<IdentityPoolEntry> {
    let proxy_url = trusted_ingress_proxy_url_for_client_ip(config, client_ip)?;
    Some(IdentityPoolEntry {
        label: label.trim().to_string(),
        proxy_url,
        identity_class: identity_class.trim().to_string(),
        country_code: country_code.trim().to_ascii_uppercase(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        fixed_proxy_entry_for_client_ip, trusted_ingress_proxy_config_from_env,
        trusted_ingress_proxy_url_for_client_ip, TrustedIngressProxyConfig,
    };

    #[test]
    fn trusted_ingress_proxy_url_encodes_client_ip_in_proxy_userinfo() {
        let config = TrustedIngressProxyConfig {
            base_proxy_url: "http://127.0.0.1:3871".to_string(),
            auth_token: "ingress token".to_string(),
        };

        let proxy_url =
            trusted_ingress_proxy_url_for_client_ip(&config, "198.51.100.44").expect("proxy url");

        assert_eq!(
            proxy_url,
            "http://198.51.100.44:ingress%20token@127.0.0.1:3871"
        );
    }

    #[test]
    fn trusted_ingress_proxy_url_rejects_invalid_client_ip() {
        let config = TrustedIngressProxyConfig {
            base_proxy_url: "http://127.0.0.1:3871".to_string(),
            auth_token: "ingress-token".to_string(),
        };

        assert!(trusted_ingress_proxy_url_for_client_ip(&config, "not-an-ip").is_none());
    }

    #[test]
    fn trusted_ingress_proxy_config_from_env_requires_valid_base_url_and_token() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN");
        assert!(trusted_ingress_proxy_config_from_env().is_none());

        std::env::set_var(
            "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL",
            "http://127.0.0.1:3871",
        );
        std::env::set_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN", "ingress-token");
        assert_eq!(
            trusted_ingress_proxy_config_from_env(),
            Some(TrustedIngressProxyConfig {
                base_proxy_url: "http://127.0.0.1:3871".to_string(),
                auth_token: "ingress-token".to_string(),
            })
        );

        std::env::set_var(
            "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL",
            "http://127.0.0.1:3871/path",
        );
        assert!(trusted_ingress_proxy_config_from_env().is_none());
    }

    #[test]
    fn fixed_proxy_entry_for_client_ip_preserves_identity_metadata() {
        let config = TrustedIngressProxyConfig {
            base_proxy_url: "http://127.0.0.1:3871".to_string(),
            auth_token: "ingress-token".to_string(),
        };

        let entry = fixed_proxy_entry_for_client_ip(
            &config,
            "198.51.100.44",
            "trusted-ingress",
            "datacenter",
            "us",
        )
        .expect("fixed proxy entry");

        assert_eq!(entry.label, "trusted-ingress");
        assert_eq!(entry.identity_class, "datacenter");
        assert_eq!(entry.country_code, "US");
        assert_eq!(
            entry.proxy_url,
            "http://198.51.100.44:ingress-token@127.0.0.1:3871"
        );
    }
}
