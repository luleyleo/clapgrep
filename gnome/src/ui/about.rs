use gtk::License;

static RELEASE_NOTES: &str = r#"
<p>Bug fixes:</p>
<ul>
    <li>Fix translations for Flatpak.</li>
    <li>Fix version in about dialog.</li>
</ul>
"#;

pub fn about_dialog() -> adw::AboutDialog {
    adw::AboutDialog::builder()
        .application_name("Clapgrep")
        .version("1.3.1")
        .release_notes(RELEASE_NOTES)
        .application_icon(crate::APP_ID)
        .developer_name("Leopold Luley")
        .website("https://github.com/luleyleo/clapgrep")
        .issue_url("https://github.com/luleyleo/clapgrep/issues")
        .license_type(License::Gpl30)
        .copyright("© 2024 Leopold Luley")
        .build()
}
