use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DifficultyPolicy {
    pub min: u8,
    pub max: u8,
    pub base: u8,
    pub adaptive: bool,
}

impl DifficultyPolicy {
    pub(crate) fn bounded(self) -> Self {
        let min = self.min.clamp(4, 26);
        let max = self.max.clamp(min, 28);
        let base = self.base.clamp(min, max);
        Self {
            min,
            max,
            base,
            adaptive: self.adaptive,
        }
    }
}

pub(crate) fn verify_hashcash(raw_token: &str, nonce: &str, difficulty: u8) -> bool {
    if difficulty == 0 {
        return true;
    }
    if nonce.trim().is_empty() || nonce.len() > 256 {
        return false;
    }

    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    hasher.update(b":");
    hasher.update(nonce.as_bytes());
    let digest = hasher.finalize();

    let mut bits_remaining = difficulty;
    for byte in digest {
        if bits_remaining == 0 {
            return true;
        }
        if bits_remaining >= 8 {
            if byte != 0 {
                return false;
            }
            bits_remaining -= 8;
        } else {
            let mask = 0xff << (8 - bits_remaining);
            return (byte & mask) == 0;
        }
    }
    true
}

pub(crate) fn adaptive_difficulty(
    policy: DifficultyPolicy,
    step: u16,
    global_pressure: f32,
    bucket_pressure: f32,
) -> u8 {
    let policy = policy.bounded();
    if !policy.adaptive {
        return policy.base;
    }

    let mut next = i32::from(policy.base);
    let pressure = global_pressure.max(bucket_pressure);
    if pressure >= 0.90 {
        next += 2;
    } else if pressure >= 0.70 {
        next += 1;
    } else if pressure <= 0.20 {
        next -= 1;
    }

    // Slowly raise work for very persistent flows, but keep strict bounds.
    next += i32::from((step / 8).min(3));

    next.clamp(i32::from(policy.min), i32::from(policy.max)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashcash_verifier_rejects_missing_nonce() {
        assert!(!verify_hashcash("token", "", 10));
    }

    #[test]
    fn adaptive_difficulty_respects_bounds() {
        let policy = DifficultyPolicy {
            min: 10,
            max: 14,
            base: 12,
            adaptive: true,
        };
        assert_eq!(adaptive_difficulty(policy, 0, 0.0, 0.0), 11);
        assert_eq!(adaptive_difficulty(policy, 0, 0.95, 0.1), 14);
        assert_eq!(adaptive_difficulty(policy, 64, 0.95, 0.95), 14);
    }
}
