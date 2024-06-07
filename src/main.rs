#![warn(clippy::pedantic)]
// Enable some from restriction category
#![warn(
    clippy::alloc_instead_of_core,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::deref_by_slicing,
    clippy::disallowed_script_idents,
    clippy::exit,
    clippy::expect_used,
    clippy::filetype_is_file,
    clippy::if_then_some_else_none,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::panic,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::todo,
    clippy::unimplemented,
    clippy::unneeded_field_pattern
)]
// Without self method can be called without object, which is undesirable
#![allow(clippy::unused_self)]
// Preferring explicit new call instead of Default::default
#![allow(clippy::new_without_default)]

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
