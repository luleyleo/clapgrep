use gettextrs::{gettext, ngettext};
use std::path::PathBuf;

pub fn setup_gettext() {
    let mut text_domain = gettextrs::TextDomain::new(crate::APP_ID);

    if cfg!(debug_assertions) {
        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("build");
        log::debug!("Looking for PO files in {assets_dir:?}");
        text_domain = text_domain.push(assets_dir);
    }

    if let Err(error) = text_domain.init() {
        log::error!("Failed to setup gettext: {}", error);
    };
}

/// Like `gettext`, but replaces named variables with the given dictionary.
///
/// The expected format to replace is `{name}`, where `name` is the first string
/// in the dictionary entry tuple.
pub fn gettext_f(msgid: &str, args: &[(&str, &str)]) -> String {
    let s = gettext(msgid);
    freplace(s, args)
}

/// Like `ngettext`, but replaces named variables with the given dictionary.
///
/// The expected format to replace is `{name}`, where `name` is the first string
/// in the dictionary entry tuple.
#[allow(unused)]
pub fn ngettext_f(msgid: &str, msgid_plural: &str, n: u32, args: &[(&str, &str)]) -> String {
    let s = ngettext(msgid, msgid_plural, n);
    freplace(s, args)
}

fn freplace(s: String, args: &[(&str, &str)]) -> String {
    let mut s = s;

    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }

    s
}
