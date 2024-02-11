use iced::{
    color, theme,
    widget::{button, column, container, image, svg, text, text_input},
    Application, Background, BorderRadius, Length, Theme,
};

use crate::app::{App, Message, PasswordInput};

type Element<'a> = iced::Element<'a, <App as Application>::Message>;

impl App {
    pub fn view(&self) -> Element {
        container(self.panel())
            .center_x()
            .center_y()
            .height(Length::Fill)
            .width(Length::Fill)
            .style(|_: &_| container::Appearance {
                background: Some(Background::Color(color!(0x000000))),
                ..Default::default()
            })
            .into()
    }

    fn panel(&self) -> Element {
        container(
            column![
                self.user_image(),
                self.password_input(),
                self.unlock_button(),
            ]
            .spacing(30)
            .align_items(iced::Alignment::Center),
        )
        .padding([50, 100])
        .max_width(600)
        .style(|theme: &Theme| container::Appearance {
            background: Some(Background::Color(theme.palette().background)),
            border_radius: BorderRadius::from(10.0),
            ..Default::default()
        })
        .into()
    }

    fn password_input(&self) -> Element {
        text_input("", &self.password)
            .password()
            .id(self.password_input.clone())
            .on_input(|val| Message::PasswordInput(PasswordInput::Value(val)))
            .on_submit(Message::PasswordInput(PasswordInput::Submit))
            .into()
    }

    fn unlock_button(&self) -> Element {
        button(text("Unlock")).on_press(Message::Unlock).into()
    }

    fn user_image(&self) -> Element {
        if let Some(user_image) = &self.user_image {
            image(user_image.clone())
                .border_radius([50.0, 50.0, 50.0, 50.0])
                .width(100)
                .height(100)
                .into()
        } else {
            svg(self.placeholder_user_image.clone())
                .width(100)
                .height(100)
                .style(theme::Svg::custom_fn(|_theme| svg::Appearance {
                    color: Some(color!(0x7aa2f7)),
                }))
                .into()
        }
    }
}
