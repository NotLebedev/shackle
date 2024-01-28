mod signal_handler;

use crate::signal_handler::signal_command;
use iced::event::listen_raw;
use iced::wayland::session_lock;
use iced::widget::{button, column, container};
use iced::Length;
use iced::{
    event::wayland::{Event as WaylandEvent, OutputEvent, SessionLockEvent},
    wayland::InitialSurface,
    widget::text,
    window, Application, Command, Element, Subscription, Theme,
};
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
struct Locker {}

#[derive(Debug, Clone)]
pub enum Message {
    WaylandEvent(WaylandEvent),
    Unlock,
    Ignore,
}

impl Application for Locker {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Locker, Command<Self::Message>) {
        (Locker {}, session_lock::lock())
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
        container(column![unlock_button])
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
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
        Theme::Dark
    }
}