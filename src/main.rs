mod auth;
mod signal_handler;

use crate::auth::start_password_check;
use crate::signal_handler::signal_command;
use auth::check_password;
use iced::event::listen_raw;
use iced::wayland::session_lock;
use iced::widget::{button, column, container, text_input};
use iced::{
    event::wayland::{Event as WaylandEvent, OutputEvent, SessionLockEvent},
    wayland::InitialSurface,
    widget::text,
    window, Application, Command, Element, Subscription, Theme,
};
use iced::{theme, Length};
use iced_runtime::window::Id as SurfaceId;
use log::info;

fn main() {
    let settings = iced::Settings {
        initial_surface: InitialSurface::None,
        ..Default::default()
    };

    env_logger::init();
    Locker::run(settings).unwrap();
}

#[derive(Debug, Clone, Default)]
struct Locker {
    password: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    WaylandEvent(WaylandEvent),
    PasswordInput(PasswordInput),
    Unlock,
    WrongPassword,
    Ignore,
}

#[derive(Debug, Clone)]
pub enum PasswordInput {
    Value(String),
    Submit,
}

impl Application for Locker {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Locker, Command<Self::Message>) {
        (Locker::default(), session_lock::lock())
    }

    fn title(&self, _id: window::Id) -> String {
        String::from("shackle")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::WaylandEvent(evt) => match evt {
                WaylandEvent::Output(evt, output) => match evt {
                    OutputEvent::Created(_) => {
                        info!("New output created. Initializing lock surface.");
                        return session_lock::get_lock_surface(window::Id::unique(), output);
                    }
                    _ => {}
                },
                WaylandEvent::SessionLock(evt) => match evt {
                    SessionLockEvent::Locked => {
                        return signal_command();
                    }
                    SessionLockEvent::Unlocked => {
                        info!("Session unlocked. Exiting.");
                        std::process::exit(0);
                    }
                    _ => {}
                },
                _ => {}
            },
            Message::PasswordInput(input) => match input {
                PasswordInput::Value(val) => self.password = val,
                PasswordInput::Submit => {
                    info!("Checking password \"{}\"", self.password);
                    return start_password_check(&self.password);
                }
            },
            Message::WrongPassword => {
                self.password = "".into();
            }
            Message::Unlock => {
                info!("Unlocking session.");
                return session_lock::unlock();
            }
            Message::Ignore => {}
        }
        Command::none()
    }

    fn view(&self, _id: window::Id) -> Element<Self::Message> {
        let unlock_button = button(text("Unlock")).on_press(Message::Unlock);
        let password_input = text_input("", &self.password)
            .password()
            .on_input(|val| Message::PasswordInput(PasswordInput::Value(val)))
            .on_submit(Message::PasswordInput(PasswordInput::Submit));
        container(
            column![password_input, unlock_button]
                .align_items(iced::Alignment::Center)
                .max_width(800),
        )
        .center_x()
        .center_y()
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        listen_raw(|evt, _| {
            if let iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(evt)) = evt
            {
                Some(Message::WaylandEvent(evt))
            } else {
                None
            }
        })
    }

    fn theme(&self, _id: SurfaceId) -> Self::Theme {
        Theme::custom(theme::Palette {
            background: iced::Color::from_rgb8(0x1a, 0x1b, 0x26),
            text: iced::Color::from_rgb8(0xc0, 0xca, 0xf5),
            primary: iced::Color::from_rgb8(0x2a, 0xc3, 0xde),
            success: iced::Color::from_rgb8(0x9e, 0xce, 0x6a),
            danger: iced::Color::from_rgb8(0xdb, 0x4b, 0x4b),
        })
    }
}
