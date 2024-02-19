use app::App;
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
        if let Ok(Fork::Child) = daemon(false, false) {
            start();
        }
    } else {
        start();
    }
}

fn start() {
    env_logger::init();
    let settings = App::build_settings();
    App::run(settings).unwrap();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    daemonize: bool,
}
