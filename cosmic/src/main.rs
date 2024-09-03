// SPDX-License-Identifier: {{LICENSE}}

mod app;
mod config;

fn main() -> cosmic::iced::Result {
    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default();

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, ())
}
