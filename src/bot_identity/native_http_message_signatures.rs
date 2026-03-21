use serde::{Deserialize, Serialize};
use sfv::SerializeValue;
use sha2::{Digest, Sha256};
use spin_sdk::http::Request;
use std::time::{SystemTime, UNIX_EPOCH};
use web_bot_auth::components::{CoveredComponent, DerivedComponent, HTTPField, HTTPFieldParameters};
use web_bot_auth::keyring::{JSONWebKeySet, KeyRing};
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
const VERIFIED_IDENTITY_DIRECTORY_CACHE_PREFIX: &str = "verified_identity:directory_cache";
const VERIFIED_IDENTITY_DIRECTORY_CACHE_INDEX_PREFIX: &str =
    "verified_identity:directory_cache_index";
const MAX_EXTERNAL_SIGNATURE_AGENT_LINKS_PER_REQUEST: usize = 4;
const MAX_CACHED_EXTERNAL_DIRECTORIES_PER_SITE: usize = 64;
const MAX_EXTERNAL_DIRECTORY_RESPONSE_BYTES: usize = 64 * 1024;

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
        store: &dyn crate::challenge::KeyValueStore,
        site_id: &str,
        cfg: &crate::config::Config,
        now_secs: u64,
        verifier: &WebBotAuthVerifier,
    ) -> Result<ResolvedNativeIdentity, IdentityVerificationFailure>;
}

trait DirectoryFetcher {
    fn fetch(&self, uri: &str) -> Result<DirectoryFetchResult, ()>;
}

struct DirectoryFetchResult {
    status: u16,
    body: Vec<u8>,
}

#[derive(Default)]
struct SpinDirectoryFetcher;

impl DirectoryFetcher for SpinDirectoryFetcher {
    fn fetch(&self, uri: &str) -> Result<DirectoryFetchResult, ()> {
        dispatch_directory_fetch(uri)
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct CachedExternalDirectoryRecord {
    source_uri: String,
    fetched_at: u64,
    jwks: JSONWebKeySet,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
struct CachedExternalDirectoryIndex {
    entries: Vec<CachedExternalDirectoryIndexEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct CachedExternalDirectoryIndexEntry {
    source_uri: String,
    fetched_at: u64,
}

struct BoundedDirectoryResolver<'a> {
    fetcher: &'a dyn DirectoryFetcher,
}

struct InlineOnlyDirectoryResolver;

impl NativeDirectoryResolver for InlineOnlyDirectoryResolver {
    fn resolve(
        &self,
        _store: &dyn crate::challenge::KeyValueStore,
        _site_id: &str,
        _cfg: &crate::config::Config,
        _now_secs: u64,
        verifier: &WebBotAuthVerifier,
    ) -> Result<ResolvedNativeIdentity, IdentityVerificationFailure> {
        for link in verifier.get_signature_agents() {
            match link {
                SignatureAgentLink::Inline(jwks) => {
                    return build_inline_resolved_identity(verifier, jwks.clone());
                }
                SignatureAgentLink::External(_) => continue,
            }
        }

        Err(IdentityVerificationFailure::DirectoryUnavailable)
    }
}

impl NativeDirectoryResolver for BoundedDirectoryResolver<'_> {
    fn resolve(
        &self,
        store: &dyn crate::challenge::KeyValueStore,
        site_id: &str,
        cfg: &crate::config::Config,
        now_secs: u64,
        verifier: &WebBotAuthVerifier,
    ) -> Result<ResolvedNativeIdentity, IdentityVerificationFailure> {
        let mut saw_signature_invalid = false;
        let mut saw_directory_stale = false;
        let mut saw_directory_unavailable = false;

        for link in verifier
            .get_signature_agents()
            .iter()
            .take(MAX_EXTERNAL_SIGNATURE_AGENT_LINKS_PER_REQUEST)
        {
            match link {
                SignatureAgentLink::Inline(jwks) => {
                    return build_inline_resolved_identity(verifier, jwks.clone());
                }
                SignatureAgentLink::External(uri) => match resolve_external_identity(
                    store,
                    site_id,
                    cfg,
                    now_secs,
                    uri.as_str(),
                    self.fetcher,
                ) {
                    Ok(resolved) => return Ok(resolved),
                    Err(IdentityVerificationFailure::SignatureInvalid) => {
                        saw_signature_invalid = true;
                    }
                    Err(IdentityVerificationFailure::DirectoryStale) => {
                        saw_directory_stale = true;
                    }
                    Err(IdentityVerificationFailure::DirectoryUnavailable) => {
                        saw_directory_unavailable = true;
                    }
                    Err(other) => return Err(other),
                },
            }
        }

        if saw_signature_invalid {
            Err(IdentityVerificationFailure::SignatureInvalid)
        } else if saw_directory_stale {
            Err(IdentityVerificationFailure::DirectoryStale)
        } else if saw_directory_unavailable || !verifier.get_signature_agents().is_empty() {
            Err(IdentityVerificationFailure::DirectoryUnavailable)
        } else {
            Err(IdentityVerificationFailure::MissingAssertion)
        }
    }
}

pub(crate) fn verify_request(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    req: &Request,
    cfg: &crate::config::Config,
) -> IdentityVerificationResult {
    let fetcher = SpinDirectoryFetcher;
    let resolver = BoundedDirectoryResolver { fetcher: &fetcher };
    verify_request_with_now_and_resolver(
        store,
        site_id,
        req,
        cfg,
        current_unix_timestamp(),
        &resolver,
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

    let resolved = match resolver.resolve(store, site_id, cfg, now_secs, &verifier) {
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

fn build_inline_resolved_identity(
    verifier: &WebBotAuthVerifier,
    jwks: JSONWebKeySet,
) -> Result<ResolvedNativeIdentity, IdentityVerificationFailure> {
    let key_id = verifier
        .get_parsed_label()
        .base
        .parameters
        .details
        .keyid
        .clone()
        .ok_or(IdentityVerificationFailure::MissingAssertion)?;

    Ok(ResolvedNativeIdentity {
        keyring: keyring_from_jwks(jwks)?,
        stable_identity: key_id.clone(),
        operator: "inline_directory".to_string(),
        category: IdentityCategory::Other,
        end_user_controlled: false,
        directory_source: Some(IdentityDirectorySource {
            source_id: format!("inline-jwks:{key_id}"),
            source_uri: None,
        }),
    })
}

fn keyring_from_jwks(jwks: JSONWebKeySet) -> Result<KeyRing, IdentityVerificationFailure> {
    let mut keyring = KeyRing::default();
    let import_results = keyring.import_jwks(jwks);
    if import_results.iter().all(|result| result.is_some()) {
        return Err(IdentityVerificationFailure::SignatureInvalid);
    }
    Ok(keyring)
}

fn resolve_external_identity(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    cfg: &crate::config::Config,
    now_secs: u64,
    raw_uri: &str,
    fetcher: &dyn DirectoryFetcher,
) -> Result<ResolvedNativeIdentity, IdentityVerificationFailure> {
    let normalized_uri =
        normalize_https_directory_uri(raw_uri).ok_or(IdentityVerificationFailure::DirectoryUnavailable)?;
    let cached = load_cached_external_directory(store, site_id, normalized_uri.as_str());
    if let Some(record) = cached.as_ref() {
        let age = now_secs.saturating_sub(record.fetched_at);
        let within_freshness = age <= cfg.verified_identity.directory_freshness_requirement_seconds;
        let within_direct_use = age <= cfg.verified_identity.directory_cache_ttl_seconds && within_freshness;
        if within_direct_use {
            return build_external_resolved_identity(normalized_uri.as_str(), record.jwks.clone());
        }
    }

    match fetch_external_directory(fetcher, normalized_uri.as_str()) {
        Ok(jwks) => {
            persist_cached_external_directory(store, site_id, normalized_uri.as_str(), now_secs, &jwks);
            build_external_resolved_identity(normalized_uri.as_str(), jwks)
        }
        Err(()) => match cached {
            Some(record)
                if now_secs.saturating_sub(record.fetched_at)
                    <= cfg.verified_identity.directory_freshness_requirement_seconds =>
            {
                build_external_resolved_identity(normalized_uri.as_str(), record.jwks)
            }
            Some(_) => Err(IdentityVerificationFailure::DirectoryStale),
            None => Err(IdentityVerificationFailure::DirectoryUnavailable),
        },
    }
}

fn build_external_resolved_identity(
    normalized_uri: &str,
    jwks: JSONWebKeySet,
) -> Result<ResolvedNativeIdentity, IdentityVerificationFailure> {
    let authority = absolute_uri_authority(normalized_uri)
        .ok_or(IdentityVerificationFailure::DirectoryUnavailable)?;
    Ok(ResolvedNativeIdentity {
        keyring: keyring_from_jwks(jwks)?,
        stable_identity: normalized_uri.to_string(),
        operator: host_without_port(authority.as_str()),
        category: IdentityCategory::Other,
        end_user_controlled: false,
        directory_source: Some(IdentityDirectorySource {
            source_id: directory_source_id(normalized_uri),
            source_uri: Some(normalized_uri.to_string()),
        }),
    })
}

fn fetch_external_directory(
    fetcher: &dyn DirectoryFetcher,
    uri: &str,
) -> Result<JSONWebKeySet, ()> {
    let response = fetcher.fetch(uri)?;
    if response.status != 200 || response.body.len() > MAX_EXTERNAL_DIRECTORY_RESPONSE_BYTES {
        return Err(());
    }
    serde_json::from_slice::<JSONWebKeySet>(response.body.as_slice()).map_err(|_| ())
}

fn load_cached_external_directory(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    source_uri: &str,
) -> Option<CachedExternalDirectoryRecord> {
    let cache_key = external_directory_cache_key(site_id, source_uri);
    let raw = store.get(cache_key.as_str()).ok().flatten()?;
    match serde_json::from_slice::<CachedExternalDirectoryRecord>(raw.as_slice()) {
        Ok(record) if record.source_uri == source_uri => Some(record),
        _ => {
            let _ = store.delete(cache_key.as_str());
            None
        }
    }
}

fn persist_cached_external_directory(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    source_uri: &str,
    fetched_at: u64,
    jwks: &JSONWebKeySet,
) {
    let record = CachedExternalDirectoryRecord {
        source_uri: source_uri.to_string(),
        fetched_at,
        jwks: jwks.clone(),
    };
    let Ok(raw_record) = serde_json::to_vec(&record) else {
        return;
    };
    let cache_key = external_directory_cache_key(site_id, source_uri);
    if store.set(cache_key.as_str(), raw_record.as_slice()).is_err() {
        return;
    }

    let mut index = load_cached_external_directory_index(store, site_id);
    index.entries.retain(|entry| entry.source_uri != source_uri);
    index.entries.push(CachedExternalDirectoryIndexEntry {
        source_uri: source_uri.to_string(),
        fetched_at,
    });
    index.entries.sort_by_key(|entry| entry.fetched_at);

    while index.entries.len() > MAX_CACHED_EXTERNAL_DIRECTORIES_PER_SITE {
        let evicted = index.entries.remove(0);
        let evicted_key = external_directory_cache_key(site_id, evicted.source_uri.as_str());
        let _ = store.delete(evicted_key.as_str());
    }

    if persist_cached_external_directory_index(store, site_id, &index).is_err() {
        let _ = store.delete(cache_key.as_str());
    }
}

fn load_cached_external_directory_index(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
) -> CachedExternalDirectoryIndex {
    let index_key = external_directory_cache_index_key(site_id);
    let Ok(raw) = store.get(index_key.as_str()) else {
        return CachedExternalDirectoryIndex::default();
    };
    let Some(raw) = raw else {
        return rebuild_cached_external_directory_index(store, site_id);
    };
    match serde_json::from_slice::<CachedExternalDirectoryIndex>(raw.as_slice()) {
        Ok(index) => index,
        Err(_) => {
            let _ = store.delete(index_key.as_str());
            rebuild_cached_external_directory_index(store, site_id)
        }
    }
}

fn persist_cached_external_directory_index(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    index: &CachedExternalDirectoryIndex,
) -> Result<(), ()> {
    let raw = serde_json::to_vec(index).map_err(|_| ())?;
    store.set(external_directory_cache_index_key(site_id).as_str(), raw.as_slice())
}

fn rebuild_cached_external_directory_index(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
) -> CachedExternalDirectoryIndex {
    let prefix = format!("{VERIFIED_IDENTITY_DIRECTORY_CACHE_PREFIX}:{site_id}:");
    let Ok(keys) = store.get_keys() else {
        return CachedExternalDirectoryIndex::default();
    };
    let mut entries = Vec::new();

    for key in keys {
        if !key.starts_with(prefix.as_str()) {
            continue;
        }
        let Some(raw) = store.get(key.as_str()).ok().flatten() else {
            continue;
        };
        match serde_json::from_slice::<CachedExternalDirectoryRecord>(raw.as_slice()) {
            Ok(record) => entries.push(CachedExternalDirectoryIndexEntry {
                source_uri: record.source_uri,
                fetched_at: record.fetched_at,
            }),
            Err(_) => {
                let _ = store.delete(key.as_str());
            }
        }
    }

    entries.sort_by_key(|entry| entry.fetched_at);
    while entries.len() > MAX_CACHED_EXTERNAL_DIRECTORIES_PER_SITE {
        let evicted = entries.remove(0);
        let evicted_key = external_directory_cache_key(site_id, evicted.source_uri.as_str());
        let _ = store.delete(evicted_key.as_str());
    }

    let rebuilt = CachedExternalDirectoryIndex { entries };
    let _ = persist_cached_external_directory_index(store, site_id, &rebuilt);
    rebuilt
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

fn external_directory_cache_key(site_id: &str, source_uri: &str) -> String {
    format!(
        "{VERIFIED_IDENTITY_DIRECTORY_CACHE_PREFIX}:{site_id}:{}",
        hash_external_directory_source(source_uri)
    )
}

fn external_directory_cache_index_key(site_id: &str) -> String {
    format!("{VERIFIED_IDENTITY_DIRECTORY_CACHE_INDEX_PREFIX}:{site_id}")
}

fn hash_external_directory_source(source_uri: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_uri.as_bytes());
    format!("{:x}", hasher.finalize())
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

fn host_without_port(authority: &str) -> String {
    let trimmed = authority.trim();
    if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            return rest[..end].to_string();
        }
    }
    if trimmed.matches(':').count() == 1 {
        return trimmed.split(':').next().unwrap_or("").to_string();
    }
    trimmed.to_string()
}

fn normalize_https_directory_uri(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let (scheme_raw, remainder) = trimmed.split_once("://")?;
    if !scheme_raw.eq_ignore_ascii_case("https") || remainder.is_empty() || remainder.contains('@') {
        return None;
    }

    let cut = remainder.find(['?', '#']).unwrap_or(remainder.len());
    if cut < remainder.len() {
        return None;
    }
    let sanitized = &remainder[..cut];
    let slash_index = sanitized.find('/').unwrap_or(sanitized.len());
    let authority = sanitized[..slash_index]
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if authority.is_empty() || authority.contains(' ') {
        return None;
    }
    let path = if slash_index < sanitized.len() {
        &sanitized[slash_index..]
    } else {
        "/"
    };
    Some(format!("https://{authority}{path}"))
}

fn directory_source_id(source_uri: &str) -> String {
    format!("http-message-signatures-directory:{source_uri}")
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

#[cfg(target_arch = "wasm32")]
fn dispatch_directory_fetch(uri: &str) -> Result<DirectoryFetchResult, ()> {
    let mut builder = Request::builder();
    builder.method(spin_sdk::http::Method::Get).uri(uri);
    let request = builder.build();
    let response = spin_sdk::http::run(spin_sdk::http::send(request)).map_err(|_| ())?;
    Ok(DirectoryFetchResult {
        status: *response.status(),
        body: response.body().to_vec(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn dispatch_directory_fetch(_uri: &str) -> Result<DirectoryFetchResult, ()> {
    Err(())
}

#[cfg(test)]
fn verify_request_with_now_and_fetcher(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    req: &Request,
    cfg: &crate::config::Config,
    now_secs: u64,
    fetcher: &dyn DirectoryFetcher,
) -> IdentityVerificationResult {
    let resolver = BoundedDirectoryResolver { fetcher };
    verify_request_with_now_and_resolver(store, site_id, req, cfg, now_secs, &resolver)
}

#[cfg(test)]
fn store_cached_directory_for_tests(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    source_uri: &str,
    fetched_at: u64,
    jwks: &JSONWebKeySet,
) {
    let normalized_uri = normalize_https_directory_uri(source_uri).expect("normalized test uri");
    persist_cached_external_directory(store, site_id, normalized_uri.as_str(), fetched_at, jwks);
}

#[cfg(test)]
fn load_directory_cache_index_for_tests(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
) -> Vec<CachedExternalDirectoryIndexEntry> {
    load_cached_external_directory_index(store, site_id).entries
}

#[cfg(test)]
mod tests {
    use super::{
        external_directory_cache_index_key, load_directory_cache_index_for_tests,
        store_cached_directory_for_tests, verify_request,
        verify_request_with_now_and_fetcher, verify_request_with_now_and_resolver,
        DirectoryFetchResult, DirectoryFetcher, NativeDirectoryResolver, ResolvedNativeIdentity,
        MAX_CACHED_EXTERNAL_DIRECTORIES_PER_SITE,
    };
    use base64::{engine::general_purpose, Engine as _};
    use spin_sdk::http::Request;
    use std::cell::Cell;
    use crate::challenge::KeyValueStore;
    use web_bot_auth::components::{
        CoveredComponent, DerivedComponent, HTTPField, HTTPFieldParameters, HTTPFieldParametersSet,
    };
    use web_bot_auth::keyring::{Algorithm, JSONWebKeySet, KeyRing, Thumbprintable};
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
            _store: &dyn crate::challenge::KeyValueStore,
            _site_id: &str,
            _cfg: &crate::config::Config,
            _now_secs: u64,
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

    struct TestDirectoryFetcher {
        body: Option<Vec<u8>>,
        calls: Cell<usize>,
    }

    impl TestDirectoryFetcher {
        fn success_for_public_key() -> Self {
            let public_key = Thumbprintable::OKP {
                crv: "Ed25519".to_string(),
                x: general_purpose::URL_SAFE_NO_PAD.encode(TEST_PUBLIC_KEY),
            };
            let jwks = serde_json::json!({
                "keys": [public_key]
            })
            .to_string()
            .into_bytes();
            Self {
                body: Some(jwks),
                calls: Cell::new(0),
            }
        }

        fn unavailable() -> Self {
            Self {
                body: None,
                calls: Cell::new(0),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.get()
        }
    }

    impl DirectoryFetcher for TestDirectoryFetcher {
        fn fetch(&self, _uri: &str) -> Result<DirectoryFetchResult, ()> {
            self.calls.set(self.calls.get().saturating_add(1));
            self.body
                .clone()
                .map(|body| DirectoryFetchResult { status: 200, body })
                .ok_or(())
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
    fn external_directory_fetch_verifies_signed_requests() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = native_enabled_config();
        let req = externally_signed_request();
        let created = request_created_at(&req);
        let fetcher = TestDirectoryFetcher::success_for_public_key();

        let result = verify_request_with_now_and_fetcher(
            &store,
            "default",
            &req,
            &cfg,
            created.saturating_add(1),
            &fetcher,
        );

        assert_eq!(
            result.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::Verified
        );
        let identity = result.identity.expect("verified identity");
        assert_eq!(identity.stable_identity, "https://signature-agent.test/");
        assert_eq!(identity.operator, "signature-agent.test");
        assert_eq!(
            identity.directory_source.expect("directory source").source_uri.as_deref(),
            Some("https://signature-agent.test/")
        );
        assert_eq!(fetcher.call_count(), 1);
    }

    #[test]
    fn external_directory_refresh_failure_uses_cached_fresh_directory() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = native_enabled_config();
        let req = externally_signed_request();
        let created = request_created_at(&req);
        let fresh_jwks = successful_test_jwks();
        let fetched_at = created.saturating_sub(cfg.verified_identity.directory_cache_ttl_seconds + 1);
        store_cached_directory_for_tests(
            &store,
            "default",
            TEST_SIGNATURE_AGENT_URL,
            fetched_at,
            &fresh_jwks,
        );
        let fetcher = TestDirectoryFetcher::unavailable();

        let result = verify_request_with_now_and_fetcher(
            &store,
            "default",
            &req,
            &cfg,
            created,
            &fetcher,
        );

        assert_eq!(
            result.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::Verified
        );
        assert_eq!(fetcher.call_count(), 1);
    }

    #[test]
    fn external_directory_refresh_failure_reports_stale_when_cache_exceeds_freshness_requirement() {
        let store = crate::test_support::InMemoryStore::default();
        let cfg = native_enabled_config();
        let req = externally_signed_request();
        let created = request_created_at(&req);
        let fresh_jwks = successful_test_jwks();
        let fetched_at =
            created.saturating_sub(cfg.verified_identity.directory_freshness_requirement_seconds + 1);
        store_cached_directory_for_tests(
            &store,
            "default",
            TEST_SIGNATURE_AGENT_URL,
            fetched_at,
            &fresh_jwks,
        );
        let fetcher = TestDirectoryFetcher::unavailable();

        let result = verify_request_with_now_and_fetcher(
            &store,
            "default",
            &req,
            &cfg,
            created,
            &fetcher,
        );

        assert_eq!(
            result.failure,
            Some(crate::bot_identity::verification::IdentityVerificationFailure::DirectoryStale)
        );
        assert_eq!(
            result.freshness,
            crate::bot_identity::verification::IdentityVerificationFreshness::Fresh
        );
        assert_eq!(fetcher.call_count(), 1);
    }

    #[test]
    fn external_directory_cache_eviction_keeps_bounded_entry_count() {
        let store = crate::test_support::InMemoryStore::default();
        let jwks = successful_test_jwks();

        for index in 0..=(MAX_CACHED_EXTERNAL_DIRECTORIES_PER_SITE as u64) {
            let uri = format!("https://directory-{index}.example/.well-known/http-message-signatures-directory");
            store_cached_directory_for_tests(&store, "default", uri.as_str(), 100 + index, &jwks);
        }

        let index_entries = load_directory_cache_index_for_tests(&store, "default");
        assert_eq!(index_entries.len(), MAX_CACHED_EXTERNAL_DIRECTORIES_PER_SITE);
        assert!(
            index_entries
                .iter()
                .all(|entry| entry.source_uri != "https://directory-0.example/.well-known/http-message-signatures-directory")
        );
    }

    #[test]
    fn external_directory_cache_rebuilds_index_when_missing_before_eviction() {
        let store = crate::test_support::InMemoryStore::default();
        let jwks = successful_test_jwks();

        for index in 0..(MAX_CACHED_EXTERNAL_DIRECTORIES_PER_SITE as u64) {
            let uri = format!("https://directory-{index}.example/.well-known/http-message-signatures-directory");
            store_cached_directory_for_tests(&store, "default", uri.as_str(), 100 + index, &jwks);
        }
        store
            .delete(external_directory_cache_index_key("default").as_str())
            .expect("delete cache index");

        store_cached_directory_for_tests(
            &store,
            "default",
            "https://directory-64.example/.well-known/http-message-signatures-directory",
            200,
            &jwks,
        );

        let index_entries = load_directory_cache_index_for_tests(&store, "default");
        assert_eq!(index_entries.len(), MAX_CACHED_EXTERNAL_DIRECTORIES_PER_SITE);
        assert!(
            index_entries
                .iter()
                .all(|entry| entry.source_uri != "https://directory-0.example/.well-known/http-message-signatures-directory")
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

    fn successful_test_jwks() -> JSONWebKeySet {
        JSONWebKeySet {
            keys: vec![Thumbprintable::OKP {
                crv: "Ed25519".to_string(),
                x: general_purpose::URL_SAFE_NO_PAD.encode(TEST_PUBLIC_KEY),
            }],
        }
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
