use std::fs;
use std::fs::DirEntry;
use std::path::Path;

use gtk::gdk;
use gtk::gio;
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk4_session_lock::Instance as SessionLockInstance;
use itertools::Itertools;
use log::error;
use log::info;
use rand::seq::IndexedRandom;

use crate::auth::pam::check_password;
use crate::config::config;

const CSS_SOURCE: &'static str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

pub fn load_css() {
    if let Some(display) = gdk::Display::default() {
        let css = gtk::CssProvider::new();
        css.load_from_string(CSS_SOURCE);
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    } else {
        error!("Failed to load css, could not get gdk::Display");
    }
}

pub fn set_gtk_settings() {
    let Some(settings) = gtk::Settings::default() else {
        error!("Failed to get GTK settings");
        return;
    };

    settings.set_property("gtk-cursor-aspect-ratio", 0.04);
    settings.set_property("gtk-font-name", "Inter 12");
}

async fn control_input_activated(
    password_entry: &gtk::PasswordEntry,
    button: &gtk::Button,
    lock: &SessionLockInstance,
) {
    // Blank out controls to show that
    // auth is in progress
    password_entry.set_sensitive(false);
    button.set_sensitive(false);

    let password = password_entry.text().to_string();

    if gio::spawn_blocking(move || check_password(password))
        .await
        .unwrap_or(false)
    {
        lock.unlock();
    }

    // Clear entry, reenable and focus in case
    // user needs to reenter password
    password_entry.set_text("");
    password_entry.set_sensitive(true);
    password_entry.grab_focus();
    button.set_sensitive(true);
}

pub fn controls(lock: &SessionLockInstance) -> gtk::Widget {
    let bbox = gtk::Box::builder()
        .css_classes(["controls-window"])
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .spacing(24)
        .build();

    let password_entry = gtk::PasswordEntry::new();
    let button = gtk::Button::builder().label("Unlock").build();

    password_entry.set_placeholder_text(Some("Password"));
    password_entry.connect_show(|password_entry| {
        password_entry.grab_focus();
    });

    password_entry.connect_activate(clone!(
        #[weak]
        password_entry,
        #[weak]
        button,
        #[weak]
        lock,
        move |_| {
            glib::spawn_future_local(async move {
                control_input_activated(&password_entry, &button, &lock).await;
            });
        }
    ));

    button.connect_clicked(clone!(
        #[weak]
        password_entry,
        #[weak]
        button,
        #[weak]
        lock,
        move |_| {
            glib::spawn_future_local(async move {
                control_input_activated(&password_entry, &button, &lock).await;
            });
        }
    ));

    bbox.append(&password_entry);
    bbox.append(&button);

    bbox.into()
}

pub fn background() -> gtk::Widget {
    let bg_paintable = config()
        .background
        .as_ref()
        .and_then(|bg| load_background_paintable(&bg));

    let video_picture = gtk::Picture::new();
    video_picture.set_paintable(bg_paintable.as_ref());
    video_picture.set_content_fit(gtk::ContentFit::Cover);

    video_picture.into()
}

fn load_background_paintable(src: &Path) -> Option<gtk::gdk::Paintable> {
    let src = if src.is_dir() {
        // Pick random supported file inside the dir
        let supported_children: Vec<DirEntry> = fs::read_dir(src)
            .ok()?
            .flat_map(|entry| entry.ok())
            .filter(|child| {
                if let Some(ext) = child.path().extension().and_then(|ext| ext.to_str()) {
                    ["jpg", "jpeg", "mp4"].contains(&ext)
                } else {
                    false
                }
            })
            .collect();

        info!(
            "Available backgrounds: {:?}",
            supported_children
                .iter()
                .map(DirEntry::path)
                .map(|p| p.to_string_lossy().into_owned())
                .join(", ")
        );

        &supported_children.choose(&mut rand::rng())?.path()
    } else {
        src
    };

    info!("Using {} as background", src.to_string_lossy());

    match src.extension().and_then(|os_str| os_str.to_str()) {
        Some("jpg" | "jpeg") => gdk::Texture::from_file(&gio::File::for_path(src))
            .ok()
            .map(gdk::Texture::into),
        Some("mp4") => {
            let bg_video = gtk::MediaFile::for_file(&gio::File::for_path(src));
            bg_video.set_loop(true);
            bg_video.set_playing(true);
            Some(bg_video.into())
        }
        _ => None,
    }
}
