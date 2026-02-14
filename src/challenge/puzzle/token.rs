use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::types::ChallengeSeed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeedTokenError {
    MissingPayload,
    MissingSignature,
    InvalidPayloadEncoding,
    InvalidSignatureEncoding,
    InvalidPayloadUtf8,
    SignatureMismatch,
    InvalidPayloadJson,
    InvalidOperationEnvelope(crate::challenge::operation_envelope::EnvelopeValidationError),
}

fn get_challenge_secret() -> String {
    match std::env::var("SHUMA_CHALLENGE_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => secret,
        _ => crate::config::env_string_required("SHUMA_JS_SECRET"),
    }
}

fn sign_payload(payload: &str) -> Vec<u8> {
    let secret = get_challenge_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn verify_signature(payload: &str, sig: &[u8]) -> bool {
    let secret = get_challenge_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    mac.verify_slice(sig).is_ok()
}

pub(crate) fn make_seed_token(payload: &ChallengeSeed) -> String {
    let payload_json = serde_json::to_string(payload).unwrap();
    let sig = sign_payload(&payload_json);
    let payload_b64 = general_purpose::STANDARD.encode(payload_json.as_bytes());
    let sig_b64 = general_purpose::STANDARD.encode(sig);
    format!("{}.{}", payload_b64, sig_b64)
}

pub(crate) fn parse_seed_token(token: &str) -> Result<ChallengeSeed, SeedTokenError> {
    let mut parts = token.splitn(2, '.');
    let payload_b64 = parts.next().ok_or(SeedTokenError::MissingPayload)?;
    let sig_b64 = parts.next().ok_or(SeedTokenError::MissingSignature)?;
    let payload_bytes = general_purpose::STANDARD
        .decode(payload_b64.as_bytes())
        .map_err(|_| SeedTokenError::InvalidPayloadEncoding)?;
    let sig = general_purpose::STANDARD
        .decode(sig_b64.as_bytes())
        .map_err(|_| SeedTokenError::InvalidSignatureEncoding)?;
    let payload_json =
        String::from_utf8(payload_bytes).map_err(|_| SeedTokenError::InvalidPayloadUtf8)?;

    if !verify_signature(&payload_json, &sig) {
        return Err(SeedTokenError::SignatureMismatch);
    }

    let payload = serde_json::from_str::<ChallengeSeed>(&payload_json)
        .map_err(|_| SeedTokenError::InvalidPayloadJson)?;
    crate::challenge::operation_envelope::validate_signed_operation_envelope(
        payload.operation_id.as_str(),
        payload.flow_id.as_str(),
        payload.step_id.as_str(),
        payload.issued_at,
        payload.expires_at,
        payload.token_version,
        crate::challenge::operation_envelope::FLOW_CHALLENGE_PUZZLE,
        crate::challenge::operation_envelope::STEP_CHALLENGE_PUZZLE_SUBMIT,
    )
    .map_err(SeedTokenError::InvalidOperationEnvelope)?;

    Ok(payload)
}
