use app::App;
use fork::{daemon, Fork};
use iced::Application;

pub mod app;
pub mod auth;
pub mod dbus;
pub mod signal_handler;
pub mod ui;
pub mod user_image;

fn main() {
    if let Ok(Fork::Child) = daemon(false, false) {
        env_logger::init();
        let settings = App::build_settings();
        App::run(settings).unwrap();
    };
}
