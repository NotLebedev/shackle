use std::path::Path;

use iced::widget::image;

use crate::app::Message;

pub async fn load() -> Message {
    let Some(home) = home::home_dir() else {
        return Message::Ignore;
    };

    let home = Path::new(&home);
    let image = home.join(".face");

    let Ok(data) = tokio::fs::read(image).await else {
        return Message::Ignore;
    };

    Message::UserImageLoaded(image::Handle::from_memory(data))
}
