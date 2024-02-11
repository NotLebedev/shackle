use std::{convert::identity, future::Future};

use iced::{
    event::wayland::{self},
    theme::Palette,
    wayland::session_lock,
    widget::{image, svg, text_input},
    window, Application, Element, Settings,
};
use iced_runtime::{futures::MaybeSend, Command};
use log::info;

use crate::{auth, dbus, signal_handler, user_image};

#[derive(Debug, Clone)]
pub struct App {
    pub password: String,
    pub validating_password: bool,
    pub user_image: Option<image::Handle>,
    pub placeholder_user_image: svg::Handle,
    pub password_input: iced::id::Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    WaylandEvent(wayland::Event),
    PasswordInput(PasswordInput),
    UserImageLoaded(image::Handle),
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
                user_image: None,
                placeholder_user_image: user_image::placeholder(),
            },
            session_lock::lock(),
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
                        return session_lock::get_lock_surface(window::Id::unique(), output);
                    }
                    _ => {}
                },
                wayland::Event::SessionLock(evt) => match evt {
                    wayland::SessionLockEvent::Locked => {
                        return iced::Command::batch([
                            perform(signal_handler::sighandler()),
                            perform(user_image::load()),
                            perform(dbus::fprint()),
                        ]);
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
                        self.password = val;
                    }
                }
                PasswordInput::Submit => {
                    info!("Checking password.");
                    self.validating_password = true;
                    return perform(auth::check_password(self.password.clone()));
                }
            },
            Message::WrongPassword => {
                self.password = "".into();
                self.validating_password = false;
            }
            Message::Unlock => {
                info!("Unlocking session.");
                self.validating_password = false;
                return session_lock::unlock();
            }
            Message::UserImageLoaded(image) => {
                self.user_image = Some(image);
            }
            Message::Ignore => {}
        }
        Command::none()
    }

    fn view(&self, _id: window::Id) -> Element<Self::Message> {
        self.view()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::event::listen_raw(|evt, _| match evt {
            iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(evt)) => {
                Some(Message::WaylandEvent(evt))
            }
            _ => None,
        })
    }

    fn theme(&self, _id: iced_runtime::window::Id) -> Self::Theme {
        Self::Theme::custom(Palette {
            background: iced::color!(0x1a1b26),
            text: iced::color!(0xc0caf5),
            primary: iced::color!(0x2ac3de),
            success: iced::color!(0x9ece6a),
            danger: iced::color!(0xdb4b4b),
        })
    }
}

fn perform(
    future: impl Future<Output = Message> + 'static + MaybeSend,
) -> Command<<App as iced::Application>::Message> {
    iced::Command::perform(future, identity)
}
