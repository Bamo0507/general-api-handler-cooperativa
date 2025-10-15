use std::sync::{OnceLock, Mutex};

/// Shared lock used by integration tests to serialize Redis access.
/// Placed here so every test binary uses the same OnceLock instance.
pub static REDIS_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
