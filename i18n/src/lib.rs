// SPDX-License-Identifier: {{LICENSE}}

//! Provides localization support for this crate.

use i18n_embed::{DesktopLanguageRequester, LanguageLoader};
use rust_embed::Embed;

pub use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
pub use i18n_embed_fl::fl;

#[derive(Embed)]
#[folder = "../translations"]
struct Localizations;

/// Applies the requested language(s) to requested translations from the `fl!()` macro.
pub fn init(loader: &FluentLanguageLoader) {
    // Get the system's preferred languages.
    let requested_languages = DesktopLanguageRequester::requested_languages();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    // Enable localizations to be applied.
    if let Err(why) = i18n_embed::select(loader, &Localizations, &requested_languages) {
        eprintln!("error while loading fluent localizations: {why}");
    }
}
