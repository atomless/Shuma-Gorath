pub(crate) mod assets;
#[cfg(test)]
mod benchmark;
mod content;
pub(crate) mod covert_decoy;
mod http;
pub(crate) mod preview;
mod renders;
mod rng;
pub(crate) mod runtime;
pub(crate) mod seeds;
#[cfg(test)]
mod simulation;
pub(crate) mod state;
mod token;
mod types;

pub use http::is_maze_path;

#[cfg(test)]
mod tests {
    use super::rng::SeededRng;
    use super::*;

    #[test]
    fn test_is_maze_path() {
        assert!(is_maze_path("/trap/abc123"));
        assert!(is_maze_path("/maze/def456"));
        assert!(!is_maze_path("/admin/config"));
        assert!(!is_maze_path("/api/data"));
    }

    #[test]
    fn test_seeded_rng_deterministic() {
        let mut rng1 = SeededRng::new(12345);
        let mut rng2 = SeededRng::new(12345);

        for _ in 0..10 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_seeded_rng_different_seeds() {
        let mut rng1 = SeededRng::new(11111);
        let mut rng2 = SeededRng::new(22222);

        // Very unlikely to match
        assert_ne!(rng1.next(), rng2.next());
    }
}
