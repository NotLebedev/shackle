use glib::{self, clone};
use gtk::prelude::{BoxExt, ButtonExt, EditableExt};
use gtk4_session_lock::Instance as SessionLockInstance;

fn control_input_activated(password_entry: &gtk::PasswordEntry, lock: &SessionLockInstance) {
    if password_entry.text().as_str() == "qwe" {
        lock.unlock();
    }
}

pub fn controls(lock: &SessionLockInstance) -> gtk::Widget {
    let bbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .spacing(10)
        .build();

    let password_entry = gtk::PasswordEntry::new();
    password_entry.connect_activate(clone!(
        #[weak]
        lock,
        move |password_entry| control_input_activated(password_entry, &lock)
    ));
    bbox.append(&password_entry);

    let button = gtk::Button::builder().label("Unlock").build();
    button.connect_clicked(clone!(
        #[weak]
        password_entry,
        #[weak]
        lock,
        move |_| control_input_activated(&password_entry, &lock)
    ));
    bbox.append(&button);

    bbox.into()
}
