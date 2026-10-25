/// An RGBA color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    /// The red channel.
    pub r: f32,
    /// The green channel.
    pub g: f32,
    /// The blue channel.
    pub b: f32,
    /// The alpha channel.
    pub a: f32,
}

impl Rgba {
    /// Creates a color from normalized red, green, blue, and alpha channels.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parses a six-digit RGB hexadecimal color.
    ///
    /// The leading `#` is required and parsed colors are always fully opaque.
    ///
    /// Returns `None` for non-ASCII input, invalid hexadecimal digits or any
    /// format other than six hexadecimal digits following `#`.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#')?;
        if hex.len() != 6 || !hex.is_ascii() {
            return None;
        }
        let v = u32::from_str_radix(hex, 16).ok()?;
        let c = |shift: u32| ((v >> shift) & 0xff) as f32 / 255.0;
        Some(Self::new(c(16), c(8), c(0), 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        assert_eq!(
            Rgba::from_hex("#ff0000"),
            Some(Rgba::new(1.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(
            Rgba::from_hex("#00ff00"),
            Some(Rgba::new(0.0, 1.0, 0.0, 1.0))
        );
        assert_eq!(
            Rgba::from_hex("#0000ff"),
            Some(Rgba::new(0.0, 0.0, 1.0, 1.0))
        );

        assert!(Rgba::from_hex("#00000").is_none());
        assert!(Rgba::from_hex("ff0").is_none());
        assert!(Rgba::from_hex("#gggggg").is_none());
    }
}
