use app::{App, Flags};
use clap::Parser;
use fork::{daemon, Fork};
use iced::Application;

pub mod app;
pub mod auth;
pub mod dbus;
pub mod signal_handler;
pub mod ui;
pub mod user_image;

fn main() {
    let args = Args::parse();

    if args.daemonize {
        if let Ok(Fork::Child) = daemon(true, true) {
            start(args);
        }
    } else {
        start(args);
    }
}

fn start(args: Args) {
    env_logger::init();
    let settings = App::build_settings(Flags {
        await_wakeup: args.await_wakeup,
    });
    App::run(settings).unwrap();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Fork off locker process
    #[arg(short, long)]
    daemonize: bool,
    /// Start fingerprint verification only after device wakes up
    ///
    /// Useful if fingerprint verification does not work (or is delayed)
    /// after devices goes to sleep
    #[arg(short, long)]
    await_wakeup: bool,
}
