//! some forced stuff i am going to define.
//!   1. Hide the console window on Windows (`windows_subsystem`).
//!   2. Run the one-time startup hosts file sanitisation.
//!   3. Hand off to the Iced application.

#![cfg_attr(not(test), windows_subsystem = "windows")]

mod app;
mod blocker;
mod duration;
mod hosts;
mod permissions;

fn main() -> iced::Result {
    let hosts_path = hosts::default_hosts_path();
    if let Err(e) = hosts::sanitise_on_startup(&hosts_path) {
        eprintln!("Warning: startup hosts-file sanitisation failed: {}", e);
    }

    app::run()
}
