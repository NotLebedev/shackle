use iced::{
    widget::{button, column, container, text, text_input},
    Application, Background, BorderRadius, Color, Length,
};

use crate::app::{App, Message, PasswordInput};

type Element<'a> = iced::Element<'a, <App as Application>::Message>;

impl App {
    pub fn view(&self) -> impl Into<Element> {
        container::Container::new(self.panel().into())
            .center_x()
            .center_y()
            .height(Length::Fill)
            .width(Length::Fill)
            .style(|_: &_| container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.0, 0.0, 0.0))),
                ..Default::default()
            })
    }

    fn panel(&self) -> impl Into<Element> {
        container(
            column![self.password_input().into(), self.unlock_button().into()]
                .spacing(10)
                .align_items(iced::Alignment::Center),
        )
        .padding(100)
        .max_width(600)
        .style(|_: &_| container::Appearance {
            background: Some(Background::Color(Color::from_rgb8(0x1a, 0x1b, 0x26))),
            border_radius: BorderRadius::from(10.0),
            ..Default::default()
        })
    }

    fn password_input(&self) -> impl Into<Element> {
        text_input("", &self.password)
            .password()
            .id(self.password_input.clone())
            .on_input(|val| Message::PasswordInput(PasswordInput::Value(val)))
            .on_submit(Message::PasswordInput(PasswordInput::Submit))
    }

    fn unlock_button(&self) -> impl Into<Element> {
        button(text("Unlock")).on_press(Message::Unlock)
    }
}
