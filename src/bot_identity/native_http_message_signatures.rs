use sfv::SerializeValue;
use sha2::{Digest, Sha256};
use spin_sdk::http::Request;
use std::time::{SystemTime, UNIX_EPOCH};
use web_bot_auth::components::{CoveredComponent, DerivedComponent, HTTPField, HTTPFieldParameters};
use web_bot_auth::keyring::KeyRing;
use web_bot_auth::message_signatures::{ParameterDetails, SignedMessage};
use web_bot_auth::{ImplementationError, SignatureAgentLink, WebBotAuthVerifier};

use super::contracts::{
    IdentityCategory, IdentityDirectorySource, IdentityProvenance, IdentityScheme,
    VerificationStrength, VerifiedIdentityEvidence,
};
use super::verification::{
    IdentityVerificationFailure, IdentityVerificationFreshness, IdentityVerificationResult,
};

const VERIFIED_IDENTITY_REPLAY_PREFIX: &str = "verified_identity:replay";

#[derive(Clone)]
struct ResolvedNativeIdentity {
    keyring: KeyRing,
    stable_identity: String,
    operator: String,
    category: IdentityCategory,
    end_user_controlled: bool,
    directory_source: Option<IdentityDirectorySource>,
}

trait NativeDirectoryResolver {
    fn resolve(
        &self,
        verifier: &WebBotAuthVerifier,
    ) -> Result<ResolvedNativeIdentity, IdentityVerificationFailure>;
}

struct InlineOnlyDirectoryResolver;

impl NativeDirectoryResolver for InlineOnlyDirectoryResolver {
    fn resolve(
        &self,
        verifier: &WebBotAuthVerifier,
    ) -> Result<ResolvedNativeIdentity, IdentityVerificationFailure> {
        let key_id = verifier
            .get_parsed_label()
            .base
            .parameters
            .details
            .keyid
            .clone()
            .ok_or(IdentityVerificationFailure::MissingAssertion)?;

        for link in verifier.get_signature_agents() {
            match link {
                SignatureAgentLink::Inline(jwks) => {
                    let mut keyring = KeyRing::default();
                    let import_results = keyring.import_jwks(jwks.clone());
                    if import_results.iter().all(|result| result.is_some()) {
                        return Err(IdentityVerificationFailure::SignatureInvalid);
                    }

                    return Ok(ResolvedNativeIdentity {
                        keyring,
                        stable_identity: key_id.clone(),
                        operator: "inline_directory".to_string(),
                        category: IdentityCategory::Other,
                        end_user_controlled: false,
                        directory_source: Some(IdentityDirectorySource {
                            source_id: format!("inline-jwks:{key_id}"),
                            source_uri: None,
                        }),
                    });
                }
                SignatureAgentLink::External(_) => continue,
            }
        }

        Err(IdentityVerificationFailure::DirectoryUnavailable)
    }
}

pub(crate) fn verify_request(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    req: &Request,
    cfg: &crate::config::Config,
) -> IdentityVerificationResult {
    verify_request_with_now_and_resolver(
        store,
        site_id,
        req,
        cfg,
        current_unix_timestamp(),
        &InlineOnlyDirectoryResolver,
    )
}

fn verify_request_with_now_and_resolver(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    req: &Request,
    cfg: &crate::config::Config,
    now_secs: u64,
    resolver: &dyn NativeDirectoryResolver,
) -> IdentityVerificationResult {
    let verifier = match parse_request_verifier(req) {
        Ok(verifier) => verifier,
        Err(result) => return result,
    };

    let parameters = verifier.get_parsed_label().base.parameters.details.clone();
    let freshness = match evaluate_freshness(&parameters, now_secs, cfg) {
        Ok(freshness) => freshness,
        Err((failure, freshness)) => return IdentityVerificationResult::failed(failure, freshness),
    };

    let resolved = match resolver.resolve(&verifier) {
        Ok(resolved) => resolved,
        Err(failure) => return IdentityVerificationResult::failed(failure, freshness),
    };

    if verifier.verify(&resolved.keyring, None).is_err() {
        return IdentityVerificationResult::failed(
            IdentityVerificationFailure::SignatureInvalid,
            freshness,
        );
    }

    match enforce_replay_window(store, site_id, req, &parameters, now_secs, cfg) {
        Ok(()) => IdentityVerificationResult::verified(
            VerifiedIdentityEvidence {
                scheme: IdentityScheme::HttpMessageSignatures,
                stable_identity: resolved.stable_identity,
                operator: resolved.operator,
                category: resolved.category,
                verification_strength: VerificationStrength::Cryptographic,
                end_user_controlled: resolved.end_user_controlled,
                directory_source: resolved.directory_source,
                provenance: IdentityProvenance::Native,
            },
            freshness,
        ),
        Err(failure) => {
            IdentityVerificationResult::failed(failure, IdentityVerificationFreshness::ReplayRejected)
        }
    }
}

fn parse_request_verifier(req: &Request) -> Result<WebBotAuthVerifier, IdentityVerificationResult> {
    let has_signature = has_header(req, "signature");
    let has_signature_input = has_header(req, "signature-input");
    let has_signature_agent = has_header(req, "signature-agent");

    if !has_signature && !has_signature_input && !has_signature_agent {
        return Err(IdentityVerificationResult::not_attempted());
    }

    if !has_signature || !has_signature_input {
        return Err(IdentityVerificationResult::failed(
            IdentityVerificationFailure::MissingSignature,
            IdentityVerificationFreshness::NotApplicable,
        ));
    }

    if !has_signature_agent {
        return Err(IdentityVerificationResult::failed(
            IdentityVerificationFailure::MissingAssertion,
            IdentityVerificationFreshness::NotApplicable,
        ));
    }

    let adapter = RequestSignedMessage { req };
    WebBotAuthVerifier::parse(&adapter).map_err(map_parse_error)
}

fn map_parse_error(err: ImplementationError) -> IdentityVerificationResult {
    let failure = match err {
        ImplementationError::UnsupportedAlgorithm(_) => {
            IdentityVerificationFailure::UnsupportedScheme
        }
        ImplementationError::LookupError(CoveredComponent::HTTP(HTTPField { name, .. })) => {
            if name == "signature-agent" {
                IdentityVerificationFailure::MissingAssertion
            } else if name == "signature" || name == "signature-input" {
                IdentityVerificationFailure::MissingSignature
            } else {
                IdentityVerificationFailure::SignatureInvalid
            }
        }
        ImplementationError::ParsingError(message)
            if message.to_ascii_lowercase().contains("signature-agent") =>
        {
            IdentityVerificationFailure::MissingAssertion
        }
        ImplementationError::WebBotAuth(_) => IdentityVerificationFailure::ClockSkewRejected,
        _ => IdentityVerificationFailure::SignatureInvalid,
    };

    let freshness = if failure == IdentityVerificationFailure::ClockSkewRejected {
        IdentityVerificationFreshness::Stale
    } else {
        IdentityVerificationFreshness::NotApplicable
    };

    IdentityVerificationResult::failed(failure, freshness)
}

fn evaluate_freshness(
    parameters: &ParameterDetails,
    now_secs: u64,
    cfg: &crate::config::Config,
) -> Result<IdentityVerificationFreshness, (IdentityVerificationFailure, IdentityVerificationFreshness)> {
    let created = parameters
        .created
        .ok_or((
            IdentityVerificationFailure::MissingAssertion,
            IdentityVerificationFreshness::NotApplicable,
        ))? as i128;
    let expires = parameters
        .expires
        .ok_or((
            IdentityVerificationFailure::MissingAssertion,
            IdentityVerificationFreshness::NotApplicable,
        ))? as i128;
    let now = now_secs as i128;
    let skew = cfg.verified_identity.clock_skew_seconds as i128;

    if created > expires {
        return Err((
            IdentityVerificationFailure::ClockSkewRejected,
            IdentityVerificationFreshness::Stale,
        ));
    }

    if now >= created && now <= expires {
        return Ok(IdentityVerificationFreshness::Fresh);
    }

    if now >= created.saturating_sub(skew) && now <= expires.saturating_add(skew) {
        return Ok(IdentityVerificationFreshness::ClockSkewAccepted);
    }

    Err((
        IdentityVerificationFailure::ClockSkewRejected,
        IdentityVerificationFreshness::Stale,
    ))
}

fn enforce_replay_window(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    req: &Request,
    parameters: &ParameterDetails,
    now_secs: u64,
    cfg: &crate::config::Config,
) -> Result<(), IdentityVerificationFailure> {
    let Some(digest) = replay_digest(req, parameters) else {
        return Err(IdentityVerificationFailure::ReplayRejected);
    };
    let replay_key = replay_marker_key(site_id, digest.as_str());
    if replay_marker_seen(store, replay_key.as_str(), now_secs)
        .map_err(|_| IdentityVerificationFailure::ReplayRejected)?
    {
        return Err(IdentityVerificationFailure::ReplayRejected);
    }

    let expires = parameters
        .expires
        .unwrap_or(now_secs as i64)
        .max(0) as u64
        + cfg.verified_identity.clock_skew_seconds;
    let seen_until = expires.min(now_secs.saturating_add(cfg.verified_identity.replay_window_seconds));
    let seen_until = seen_until.max(now_secs.saturating_add(1));
    store
        .set(replay_key.as_str(), seen_until.to_string().as_bytes())
        .map_err(|_| IdentityVerificationFailure::ReplayRejected)?;
    Ok(())
}

fn replay_marker_seen(
    store: &dyn crate::challenge::KeyValueStore,
    key: &str,
    now_secs: u64,
) -> Result<bool, ()> {
    let Some(raw) = store.get(key)? else {
        return Ok(false);
    };

    let Some(raw_text) = std::str::from_utf8(raw.as_slice()).ok() else {
        return Err(());
    };
    let Some(seen_until) = raw_text.trim().parse::<u64>().ok() else {
        return Err(());
    };

    if seen_until > now_secs {
        Ok(true)
    } else {
        store.delete(key)?;
        Ok(false)
    }
}

fn replay_digest(req: &Request, parameters: &ParameterDetails) -> Option<String> {
    let signature = raw_header_values(req, "signature").join("\n");
    let signature_input = raw_header_values(req, "signature-input").join("\n");
    if signature.is_empty() || signature_input.is_empty() {
        return None;
    }

    let mut hasher = Sha256::new();
    hasher.update(signature.as_bytes());
    hasher.update(b"\0");
    hasher.update(signature_input.as_bytes());
    hasher.update(b"\0");
    hasher.update(parameters.keyid.as_deref().unwrap_or_default().as_bytes());
    Some(format!("{:x}", hasher.finalize()))
}

fn replay_marker_key(site_id: &str, digest: &str) -> String {
    format!("{VERIFIED_IDENTITY_REPLAY_PREFIX}:{site_id}:{digest}")
}

fn has_header(req: &Request, name: &str) -> bool {
    req.headers().any(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
}

fn raw_header_values(req: &Request, name: &str) -> Vec<String> {
    req.headers()
        .filter_map(|(header_name, value)| {
            if !header_name.eq_ignore_ascii_case(name) {
                return None;
            }
            value.as_str().map(ToOwned::to_owned)
        })
        .collect()
}

struct RequestSignedMessage<'a> {
    req: &'a Request,
}

impl SignedMessage for RequestSignedMessage<'_> {
    fn lookup_component(&self, name: &CoveredComponent) -> Vec<String> {
        match name {
            CoveredComponent::HTTP(field) => http_field_values(self.req, field),
            CoveredComponent::Derived(DerivedComponent::Authority { req: false }) => authority_from_request(self.req).into_iter().collect(),
            CoveredComponent::Derived(DerivedComponent::TargetUri { req: false }) => {
                target_uri_from_request(self.req).into_iter().collect()
            }
            CoveredComponent::Derived(DerivedComponent::Method { req: false }) => {
                vec![self.req.method().to_string()]
            }
            CoveredComponent::Derived(DerivedComponent::Path { req: false }) => {
                vec![self.req.path().to_string()]
            }
            CoveredComponent::Derived(DerivedComponent::Scheme { req: false }) => {
                vec![if crate::request_is_https(self.req) {
                    "https".to_string()
                } else {
                    "http".to_string()
                }]
            }
            CoveredComponent::Derived(DerivedComponent::Query { req: false }) => {
                let query = self.req.query();
                if query.is_empty() {
                    Vec::new()
                } else {
                    vec![format!("?{query}")]
                }
            }
            CoveredComponent::Derived(DerivedComponent::RequestTarget { req: false }) => {
                vec![request_target(self.req)]
            }
            _ => Vec::new(),
        }
    }
}

fn http_field_values(req: &Request, field: &HTTPField) -> Vec<String> {
    let raw_values = raw_header_values(req, field.name.as_str());
    let key = field.parameters.0.iter().find_map(|parameter| match parameter {
        HTTPFieldParameters::Key(key) => Some(key.as_str()),
        _ => None,
    });

    let Some(key) = key else {
        return raw_values;
    };

    raw_values
        .into_iter()
        .filter_map(|raw| {
            let dictionary = sfv::Parser::new(raw.as_str()).parse_dictionary().ok()?;
            let entry = dictionary.get(key)?;
            match entry {
                sfv::ListEntry::Item(item) => Some(item.serialize_value()),
                sfv::ListEntry::InnerList(_) => None,
            }
        })
        .collect()
}

fn authority_from_request(req: &Request) -> Option<String> {
    raw_header_values(req, "host")
        .into_iter()
        .next()
        .or_else(|| absolute_uri_authority(req.uri()))
}

fn target_uri_from_request(req: &Request) -> Option<String> {
    let uri = req.uri().trim();
    if uri.starts_with("http://") || uri.starts_with("https://") {
        return Some(uri.to_string());
    }

    let authority = authority_from_request(req)?;
    let scheme = if crate::request_is_https(req) { "https" } else { "http" };
    Some(format!("{scheme}://{authority}{uri}"))
}

fn absolute_uri_authority(uri: &str) -> Option<String> {
    let trimmed = uri.trim();
    let after_scheme = trimmed.split_once("://")?.1;
    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim();
    if authority.is_empty() {
        None
    } else {
        Some(authority.to_string())
    }
}

fn request_target(req: &Request) -> String {
    let mut target = req.path().to_string();
    let query = req.query();
    if !query.is_empty() {
        target.push('?');
        target.push_str(query);
    }
    target
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        verify_request, verify_request_with_now_and_resolver, NativeDirectoryResolver,
        ResolvedNativeIdentity,
    };
    use base64::{engine::general_purpose, Engine as _};
    use spin_sdk::http::Request;
    use web_bot_auth::components::{
        CoveredComponent, DerivedComponent, HTTPField, HTTPFieldParameters, HTTPFieldParametersSet,
    };
    use web_bot_auth::keyring::{Algorithm, KeyRing, Thumbprintable};
    use web_bot_auth::message_signatures::{MessageSigner, UnsignedMessage};
    use web_bot_auth::WebBotAuthVerifier;

    const TEST_KEY_ID: &str = "poqkLGiymh_W0uP6PZFw-dvez3QJT5SolqXBCW38r0U";
    const TEST_SIGNATURE_AGENT_URL: &str = "https://signature-agent.test";
    const TEST_PUBLIC_KEY: [u8; 32] = [
        0x26, 0xb4, 0x0b, 0x8f, 0x93, 0xff, 0xf3, 0xd8, 0x97, 0x11, 0x2f, 0x7e, 0xbc, 0x58,
        0x2b, 0x23, 0x2d, 0xbd, 0x72, 0x51, 0x7d, 0x08, 0x2f, 0xe8, 0x3c, 0xfb, 0x30, 0xdd,
        0xce, 0x43, 0xd1, 0xbb,
    ];
    const TEST_PRIVATE_KEY: [u8; 32] = [
        0x9f, 0x83, 0x62, 0xf8, 0x7a, 0x48, 0x4a, 0x95, 0x4e, 0x6e, 0x74, 0x0c, 0x5b, 0x4c,
        0x0e, 0x84, 0x22, 0x91, 0x39, 0xa2, 0x0a, 0xa8, 0xab, 0x56, 0xff, 0x66, 0x58, 0x6f,
        0x6a, 0x7d, 0x29, 0xc5,
    ];
    #[derive(Clone)]
    struct StaticResolver {
        keyring: KeyRing,
        stable_identity: String,
        operator: String,
        category: crate::bot_identity::contracts::IdentityCategory,
        directory_source: Option<crate::bot_identity::contracts::IdentityDirectorySource>,
    }

    impl NativeDirectoryResolver for StaticResolver {
        fn resolve(
            &self,
            _verifier: &WebBotAuthVerifier,
        ) -> Result<ResolvedNativeIdentity, crate::bot_identity::verification::IdentityVerificationFailure> {
            Ok(ResolvedNativeIdentity {
                keyring: self.keyring.clone(),
                stable_identity: self.stable_identity.clone(),
                operator: self.operator.clone(),
                category: self.category,
                end_user_controlled: true,
                directory_source: self.directory_source.clone(),
            })
        }
    }

    struct InlineSignedRequestBuilder {
        authority: String,
        signature_agent_key: String,
        signature_agent_component: String,
        signature_agent_header: String,
        signature_input: String,
        signature_header: String,
    }

    impl UnsignedMessage for InlineSignedRequestBuilder {
        fn fetch_components_to_cover(&self) -> indexmap::IndexMap<CoveredComponent, String> {
            indexmap::IndexMap::from_iter([
                (
                    CoveredComponent::Derived(DerivedComponent::Authority { req: false }),
                    self.authority.clone(),
                ),
                (
                    CoveredComponent::HTTP(HTTPField {
                        name: "signature-agent".to_string(),
                        parameters: HTTPFieldParametersSet(vec![HTTPFieldParameters::Key(
                            self.signature_agent_key.clone(),
                        )]),
                    }),
                    self.signature_agent_component.clone(),
                ),
            ])
        }

        fn register_header_contents(&mut self, signature_input: String, signature_header: String) {
            self.signature_input = format!("sig1={signature_input}");
            self.signature_header = format!("sig1={signature_header}");
        }
    }

    #[test]
    fn verify_request_reports_missing_signature_for_unsigned_signature_agent_claims() {
        let store = crate::test_support::InMemoryStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.enabled = true;
        let req = crate::test_support::request_with_headers(
            "/",
            &[("host", "example.com"), ("signature-agent", "\"https://signature-agent.test\"")],
        );

        let result = verify_request(&store, "default", &req, &cfg);

        assert_eq!(
            result.failure,
            Some(crate::bot_identity::verification::IdentityVerificationFailure::MissingSignature)
        );
        assert_eq!(
            result.freshness,
            crate::bot_identity::verification::IdentityVerificationFreshness::NotApplicable
        );
    }

    #[test]
    fn verify_request_rejects_clock_skew_outside_tolerance() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = native_enabled_config();
        let req = externally_signed_request();
        let created = request_created_at(&req);
        let resolver = spec_resolver();

        let result = verify_request_with_now_and_resolver(
            &store,
            "default",
            &req,
            &cfg,
            created.saturating_sub(100),
            &resolver,
        );

        assert_eq!(
            result.failure,
            Some(crate::bot_identity::verification::IdentityVerificationFailure::ClockSkewRejected)
        );
        assert_eq!(
            result.freshness,
            crate::bot_identity::verification::IdentityVerificationFreshness::Stale
        );
    }

    #[test]
    fn verify_request_accepts_clock_skew_inside_tolerance() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = native_enabled_config();
        let req = externally_signed_request();
        let created = request_created_at(&req);
        let resolver = spec_resolver();

        let result = verify_request_with_now_and_resolver(
            &store,
            "default",
            &req,
            &cfg,
            created.saturating_sub(20),
            &resolver,
        );

        assert_eq!(
            result.freshness,
            crate::bot_identity::verification::IdentityVerificationFreshness::ClockSkewAccepted
        );
        assert_eq!(
            result.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::Verified
        );
        assert_eq!(
            result.identity.as_ref().map(|identity| identity.operator.as_str()),
            Some("openai")
        );
    }

    #[test]
    fn verify_request_rejects_replayed_signatures_after_successful_verification() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = native_enabled_config();
        let req = externally_signed_request();
        let created = request_created_at(&req);
        let resolver = spec_resolver();

        let first = verify_request_with_now_and_resolver(
            &store,
            "default",
            &req,
            &cfg,
            created.saturating_add(1),
            &resolver,
        );
        let second = verify_request_with_now_and_resolver(
            &store,
            "default",
            &req,
            &cfg,
            created.saturating_add(2),
            &resolver,
        );

        assert_eq!(
            first.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::Verified
        );
        assert_eq!(
            second.failure,
            Some(crate::bot_identity::verification::IdentityVerificationFailure::ReplayRejected)
        );
        assert_eq!(
            second.freshness,
            crate::bot_identity::verification::IdentityVerificationFreshness::ReplayRejected
        );
    }

    #[test]
    fn verify_request_reports_directory_unavailable_for_external_signature_agents_without_resolution() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = native_enabled_config();
        let req = externally_signed_request();
        let created = request_created_at(&req);

        let result = verify_request_with_now_and_resolver(
            &store,
            "default",
            &req,
            &cfg,
            created.saturating_add(1),
            &super::InlineOnlyDirectoryResolver,
        );

        assert_eq!(
            result.failure,
            Some(crate::bot_identity::verification::IdentityVerificationFailure::DirectoryUnavailable)
        );
        assert_eq!(
            result.freshness,
            crate::bot_identity::verification::IdentityVerificationFreshness::Fresh
        );
    }

    #[test]
    fn verify_request_verifies_self_contained_inline_signature_agent_requests() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = native_enabled_config();
        let req = inline_signed_request();

        let result = verify_request(&store, "default", &req, &cfg);

        assert_eq!(
            result.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::Verified
        );
        let identity = result.identity.expect("verified identity");
        assert_eq!(
            identity.scheme,
            crate::bot_identity::contracts::IdentityScheme::HttpMessageSignatures
        );
        assert_eq!(identity.operator, "inline_directory");
        assert_eq!(identity.stable_identity, TEST_KEY_ID);
        assert_eq!(
            result.freshness,
            crate::bot_identity::verification::IdentityVerificationFreshness::Fresh
        );
    }

    fn native_enabled_config() -> crate::config::Config {
        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.enabled = true;
        cfg.verified_identity.native_web_bot_auth_enabled = true;
        cfg
    }

    fn spec_resolver() -> StaticResolver {
        let mut keyring = KeyRing::default();
        keyring.import_raw(
            TEST_KEY_ID.to_string(),
            Algorithm::Ed25519,
            TEST_PUBLIC_KEY.to_vec(),
        );
        StaticResolver {
            keyring,
            stable_identity: "chatgpt-agent".to_string(),
            operator: "openai".to_string(),
            category: crate::bot_identity::contracts::IdentityCategory::UserTriggeredAgent,
            directory_source: Some(crate::bot_identity::contracts::IdentityDirectorySource {
                source_id: "openai-http-message-signatures-directory".to_string(),
                source_uri: Some(TEST_SIGNATURE_AGENT_URL.to_string()),
            }),
        }
    }

    fn externally_signed_request() -> Request {
        signed_request_for_signature_agent(TEST_SIGNATURE_AGENT_URL, "agent2")
    }

    fn inline_signed_request() -> Request {
        let public_key = Thumbprintable::OKP {
            crv: "Ed25519".to_string(),
            x: general_purpose::URL_SAFE_NO_PAD.encode(TEST_PUBLIC_KEY),
        };
        let jwks = serde_json::json!({
            "keys": [public_key]
        })
        .to_string();
        let signature_agent_url = format!(
            "data:application/http-message-signatures-directory;base64,{}",
            general_purpose::STANDARD.encode(jwks.as_bytes())
        );
        signed_request_for_signature_agent(signature_agent_url.as_str(), "agent1")
    }

    fn signed_request_for_signature_agent(signature_agent_url: &str, signature_agent_key: &str) -> Request {
        let mut signer_message = InlineSignedRequestBuilder {
            authority: "example.com".to_string(),
            signature_agent_key: signature_agent_key.to_string(),
            signature_agent_component: format!("\"{signature_agent_url}\""),
            signature_agent_header: format!("{signature_agent_key}=\"{signature_agent_url}\""),
            signature_input: String::new(),
            signature_header: String::new(),
        };
        let signer = MessageSigner {
            keyid: TEST_KEY_ID.to_string(),
            nonce: "inline-native-verifier-test".to_string(),
            tag: "web-bot-auth".to_string(),
        };
        signer
            .generate_signature_headers_content(
                &mut signer_message,
                time::Duration::seconds(60),
                Algorithm::Ed25519,
                &TEST_PRIVATE_KEY,
            )
            .expect("signature headers");

        crate::test_support::request_with_headers(
            "/",
            &[
                ("host", "example.com"),
                ("signature-agent", signer_message.signature_agent_header.as_str()),
                ("signature-input", signer_message.signature_input.as_str()),
                ("signature", signer_message.signature_header.as_str()),
            ],
        )
    }

    fn request_created_at(req: &Request) -> u64 {
        super::parse_request_verifier(req)
            .expect("signed request should parse")
            .get_parsed_label()
            .base
            .parameters
            .details
            .created
            .expect("created timestamp") as u64
    }
}
