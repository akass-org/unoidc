use lazy_static::lazy_static;
use prometheus::{Counter, Registry};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref AUTH_REQUESTS_TOTAL: Counter = Counter::new(
        "oidc_auth_requests_total",
        "Total number of authorization requests"
    ).unwrap();

    pub static ref TOKEN_ISSUED_TOTAL: Counter = Counter::new(
        "oidc_token_issued_total",
        "Total number of tokens issued"
    ).unwrap();

    pub static ref TOKEN_REFRESH_TOTAL: Counter = Counter::new(
        "oidc_token_refresh_total",
        "Total number of token refreshes"
    ).unwrap();

    pub static ref REPLAY_DETECTED_TOTAL: Counter = Counter::new(
        "oidc_replay_detected_total",
        "Total number of replay attacks detected"
    ).unwrap();

    pub static ref SESSION_ACTIVE_TOTAL: Counter = Counter::new(
        "session_active_total",
        "Total number of active sessions"
    ).unwrap();
}

pub fn init() {
    REGISTRY.register(Box::new(AUTH_REQUESTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(TOKEN_ISSUED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(TOKEN_REFRESH_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(REPLAY_DETECTED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(SESSION_ACTIVE_TOTAL.clone())).unwrap();
}
