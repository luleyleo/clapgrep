// SPDX-License-Identifier: {{LICENSE}}

use std::sync::LazyLock;

mod app;
mod config;

static LANGUAGE_LOADER: LazyLock<clapgrep_i18n::FluentLanguageLoader> =
    LazyLock::new(|| clapgrep_i18n::fluent_language_loader!());

/// Request a localized string by ID from the i18n/ directory.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        clapgrep_i18n::fl!($crate::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        clapgrep_i18n::fl!($crate::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

fn main() -> cosmic::iced::Result {
    clapgrep_i18n::init(&LANGUAGE_LOADER);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default();

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, ())
}
