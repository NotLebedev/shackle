use gtk::glib;
use log::info;

pub async fn wait_signal() {
    glib::unix_signal_future(nix::sys::signal::Signal::SIGUSR1 as i32).await;
    info!("Recieved SIGUSR1.");
}
