use std::path::Path;

use iced::widget::{image, svg};

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

pub fn placeholder() -> svg::Handle {
    let svg = r#"
    <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
      <path fill-rule="evenodd"
        d="M 0 0 m 0 50
           a 50, 50 0 1,0  100, 0
           a 50, 50 0 1,0 -100, 0
           M 38 18
           m 0 12
           a 12, 12 0 1,0  24, 0
           a 12, 12 0 1,0 -24, 0
           M 30 80
           a 20 32 0 0 1 40 0"/>
    </svg>  
    "#;

    svg::Handle::from_memory(svg.as_bytes())
}
