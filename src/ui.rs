use gtk::gio;
use gtk::glib::{self, clone};
use gtk::prelude::{BoxExt, ButtonExt, EditableExt, WidgetExt};
use gtk4_session_lock::Instance as SessionLockInstance;

use crate::auth::pam::check_password;

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
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .spacing(10)
        .build();

    let password_entry = gtk::PasswordEntry::new();
    let button = gtk::Button::builder().label("Unlock").build();

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
