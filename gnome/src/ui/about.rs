use gtk::License;

static RELEASE_NOTES: &str = r#"
<p>New features:</p>
<ul>
    <li>The app will remember whether it should search PDF and Office files.</li>
    <li>The search backend has been completely rewritten and should be quite a bit faster.</li>
</ul>
"#;

pub fn about_dialog() -> adw::AboutDialog {
    adw::AboutDialog::builder()
        .application_name("Clapgrep")
        .version("1.2")
        .release_notes(RELEASE_NOTES)
        .application_icon(crate::APP_ID)
        .developer_name("Leopold Luley")
        .website("https://github.com/luleyleo/clapgrep")
        .issue_url("https://github.com/luleyleo/clapgrep/issues")
        .license_type(License::Gpl30)
        .copyright("© 2024 Leopold Luley")
        .build()
}
