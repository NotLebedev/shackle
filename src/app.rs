use iced::{
    event::wayland::{self},
    theme::Palette,
    widget::text_input,
    window, Application, Element, Settings,
};
use iced_runtime::Command;
use log::info;

use crate::{auth, signal_handler};

#[derive(Debug, Clone)]
pub struct App {
    pub password: String,
    pub validating_password: bool,
    pub password_input: iced::id::Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    WaylandEvent(wayland::Event),
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

impl App {
    pub fn build_settings() -> Settings<<App as Application>::Flags> {
        Settings {
            initial_surface: iced::wayland::InitialSurface::None,
            ..Default::default()
        }
    }
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced::Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                password_input: iced::id::Id::unique(),
                password: Default::default(),
                validating_password: false,
            },
            iced::wayland::session_lock::lock(),
        )
    }

    fn title(&self, _id: window::Id) -> String {
        String::from("shackle")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::WaylandEvent(evt) => match evt {
                wayland::Event::Output(evt, output) => match evt {
                    wayland::OutputEvent::Created(_) => {
                        info!("New output created. Initializing lock surface.");
                        return iced::wayland::session_lock::get_lock_surface(
                            window::Id::unique(),
                            output,
                        );
                    }
                    _ => {}
                },
                wayland::Event::SessionLock(evt) => match evt {
                    wayland::SessionLockEvent::Locked => {
                        return signal_handler::signal_command();
                    }
                    wayland::SessionLockEvent::Unlocked => {
                        info!("Session unlocked. Exiting.");
                        std::process::exit(0);
                    }
                    wayland::SessionLockEvent::Focused(..) => {
                        return text_input::focus(self.password_input.clone());
                    }
                    _ => {}
                },
                _ => {}
            },
            Message::PasswordInput(input) => match input {
                PasswordInput::Value(val) => {
                    if !self.validating_password {
                        self.password = val
                    }
                }
                PasswordInput::Submit => {
                    info!("Checking password.");
                    self.validating_password = true;
                    return auth::start_password_check(&self.password);
                }
            },
            Message::WrongPassword => {
                self.password = "".into();
                self.validating_password = false;
            }
            Message::Unlock => {
                info!("Unlocking session.");
                self.validating_password = false;
                return iced::wayland::session_lock::unlock();
            }
            Message::Ignore => {}
        }
        Command::none()
    }

    fn view(&self, _id: window::Id) -> Element<Self::Message> {
        self.view().into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::event::listen_raw(|evt, _| {
            if let iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(evt)) = evt
            {
                Some(Message::WaylandEvent(evt))
            } else {
                None
            }
        })
    }

    fn theme(&self, _id: iced_runtime::window::Id) -> Self::Theme {
        Self::Theme::custom(Palette {
            background: iced::Color::from_rgb8(0x1a, 0x1b, 0x26),
            text: iced::Color::from_rgb8(0xc0, 0xca, 0xf5),
            primary: iced::Color::from_rgb8(0x2a, 0xc3, 0xde),
            success: iced::Color::from_rgb8(0x9e, 0xce, 0x6a),
            danger: iced::Color::from_rgb8(0xdb, 0x4b, 0x4b),
        })
    }
}
