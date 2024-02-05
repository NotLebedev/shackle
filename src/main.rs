use app::App;
use iced::Application;

pub mod app;
pub mod auth;
pub mod signal_handler;
pub mod ui;
pub mod user_image;

fn main() {
    env_logger::init();
    let settings = App::build_settings();
    App::run(settings).unwrap();
}
