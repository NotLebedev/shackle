use std::sync::OnceLock;

use log::info;
use tokio::runtime::Runtime;

/// Get tokio runtime for app
/// Because gtk has its own async runtime
pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        info!("Started tokio runtime");
        Runtime::new().expect("Setting up tokio runtime needs to succeed.")
    })
}
