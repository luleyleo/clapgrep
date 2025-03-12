use gtk::{gdk, pango};

fn encode_rgba(rgba: &gdk::RGBA) -> String {
    format!(
        "#{r:04x}{g:04x}{b:04x}",
        r = (rgba.red() * (u16::MAX as f32)) as u16,
        g = (rgba.green() * (u16::MAX as f32)) as u16,
        b = (rgba.blue() * (u16::MAX as f32)) as u16,
    )
}

pub fn pango_color_from_rgba(rgba: &gdk::RGBA) -> pango::Color {
    pango::Color::parse(&encode_rgba(rgba)).unwrap()
}

#[cfg(test)]
mod test {
    use gtk::gdk::RGBA;

    use super::{encode_rgba, pango_color_from_rgba};

    #[test]
    fn test_encode_rgba() {
        let rgba = RGBA::new(1.0, 0.5, 0.0, 1.0);
        let encoding = encode_rgba(&rgba);
        assert_eq!(encoding, "#ffff7fff0000");
    }

    #[test]
    fn test_pango_color_from_rgba() {
        let rgba = RGBA::new(1.0, 0.5, 0.0, 1.0);
        let color = pango_color_from_rgba(&rgba);
        assert_eq!(color.red(), u16::MAX);
        assert_eq!(color.green(), u16::MAX / 2);
        assert_eq!(color.blue(), u16::MIN);
    }
}
