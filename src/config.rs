use std::sync::LazyLock;

use clap::Parser;

const CONFIG: LazyLock<Args> = LazyLock::new(|| Args::parse());

pub fn config() -> LazyLock<Args> {
    CONFIG
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Fork off locker process
    #[arg(short, long)]
    pub daemonize: bool,
    /// Start fingerprint verification only after device wakes up
    ///
    /// Useful if fingerprint verification does not work (or is delayed)
    /// after devices goes to sleep
    #[arg(short, long)]
    pub await_wakeup: bool,
}
