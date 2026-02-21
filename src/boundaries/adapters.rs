use spin_sdk::http::{Request, Response};

use super::contracts::{AdminBoundary, ChallengeBoundary, MazeBoundary};

pub(crate) struct DefaultChallengeBoundary;
pub(crate) struct DefaultMazeBoundary;
pub(crate) struct DefaultAdminBoundary;

impl ChallengeBoundary for DefaultChallengeBoundary {
    fn puzzle_path(&self) -> &'static str {
        crate::challenge::PUZZLE_PATH
    }

    fn not_a_bot_path(&self) -> &'static str {
        crate::challenge::NOT_A_BOT_PATH
    }

    fn render_challenge(
        &self,
        req: &Request,
        transform_count: usize,
        seed_ttl_seconds: u64,
    ) -> Response {
        crate::challenge::render_challenge_with_seed_ttl(req, transform_count, seed_ttl_seconds)
    }

    fn render_not_a_bot(&self, req: &Request, cfg: &crate::config::Config) -> Response {
        crate::challenge::render_not_a_bot(req, cfg)
    }

    fn serve_challenge_page(
        &self,
        req: &Request,
        test_mode: bool,
        transform_count: usize,
        seed_ttl_seconds: u64,
    ) -> Response {
        crate::challenge::serve_challenge_page_with_seed_ttl(
            req,
            test_mode,
            transform_count,
            seed_ttl_seconds,
        )
    }

    fn serve_not_a_bot_page(
        &self,
        req: &Request,
        test_mode: bool,
        cfg: &crate::config::Config,
    ) -> Response {
        crate::challenge::serve_not_a_bot_page(req, test_mode, cfg)
    }

    fn handle_challenge_submit_with_outcome<S: crate::challenge::KeyValueStore>(
        &self,
        store: &S,
        req: &Request,
        challenge_puzzle_attempt_window_seconds: u64,
        challenge_puzzle_attempt_limit_per_window: u32,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
        crate::challenge::handle_challenge_submit_with_outcome_with_limits(
            store,
            req,
            challenge_puzzle_attempt_window_seconds,
            challenge_puzzle_attempt_limit_per_window,
        )
    }

    fn handle_not_a_bot_submit_with_outcome<S: crate::challenge::KeyValueStore>(
        &self,
        store: &S,
        req: &Request,
        cfg: &crate::config::Config,
    ) -> crate::challenge::NotABotSubmitResult {
        crate::challenge::handle_not_a_bot_submit_with_outcome(store, req, cfg)
    }
}

impl MazeBoundary for DefaultMazeBoundary {
    fn is_maze_path(&self, path: &str) -> bool {
        crate::maze::is_maze_path(path)
    }
}

impl AdminBoundary for DefaultAdminBoundary {
    fn handle_admin(&self, req: &Request) -> Response {
        crate::admin::handle_admin(req)
    }
}

const CHALLENGE: DefaultChallengeBoundary = DefaultChallengeBoundary;
const MAZE: DefaultMazeBoundary = DefaultMazeBoundary;
const ADMIN: DefaultAdminBoundary = DefaultAdminBoundary;

pub(crate) fn challenge_puzzle_path() -> &'static str {
    CHALLENGE.puzzle_path()
}

pub(crate) fn challenge_not_a_bot_path() -> &'static str {
    CHALLENGE.not_a_bot_path()
}

pub(crate) fn render_challenge(
    req: &Request,
    transform_count: usize,
    seed_ttl_seconds: u64,
) -> Response {
    CHALLENGE.render_challenge(req, transform_count, seed_ttl_seconds)
}

pub(crate) fn render_not_a_bot(req: &Request, cfg: &crate::config::Config) -> Response {
    CHALLENGE.render_not_a_bot(req, cfg)
}

pub(crate) fn serve_challenge_page(
    req: &Request,
    test_mode: bool,
    transform_count: usize,
    seed_ttl_seconds: u64,
) -> Response {
    CHALLENGE.serve_challenge_page(req, test_mode, transform_count, seed_ttl_seconds)
}

pub(crate) fn serve_not_a_bot_page(
    req: &Request,
    test_mode: bool,
    cfg: &crate::config::Config,
) -> Response {
    CHALLENGE.serve_not_a_bot_page(req, test_mode, cfg)
}

pub(crate) fn handle_challenge_submit_with_outcome<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    challenge_puzzle_attempt_window_seconds: u64,
    challenge_puzzle_attempt_limit_per_window: u32,
) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
    CHALLENGE.handle_challenge_submit_with_outcome(
        store,
        req,
        challenge_puzzle_attempt_window_seconds,
        challenge_puzzle_attempt_limit_per_window,
    )
}

pub(crate) fn handle_not_a_bot_submit_with_outcome<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    cfg: &crate::config::Config,
) -> crate::challenge::NotABotSubmitResult {
    CHALLENGE.handle_not_a_bot_submit_with_outcome(store, req, cfg)
}

pub(crate) fn is_maze_path(path: &str) -> bool {
    MAZE.is_maze_path(path)
}

pub(crate) fn handle_admin(req: &Request) -> Response {
    ADMIN.handle_admin(req)
}
