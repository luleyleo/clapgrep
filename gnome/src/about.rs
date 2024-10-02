use gtk::License;

pub fn dialog() -> adw::AboutDialog {
    adw::AboutDialog::builder()
        .application_name("Clapgrep")
        .version("1.1")
        .application_icon(crate::APP_ID)
        .developer_name("Leopold Luley")
        .website("https://github.com/luleyleo/clapgrep")
        .issue_url("https://github.com/luleyleo/clapgrep/issues")
        .license_type(License::Gpl30)
        .copyright("© 2024 Leopold Luley")
        .build()
}
