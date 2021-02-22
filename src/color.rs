pub enum Color {
    Red,
    Blue,
    Green,
    Yellow,
    Magenta,
    Cyan,
    White,
    Black,
}

impl Color {
    /**
     * Converts a Color to a string.
     */
    pub fn to_string(&self) -> &str {
        match self {
            Color::Red => "red",
            Color::Blue => "blue",
            Color::Green => "green",
            Color::Yellow => "yellow",
            Color::Magenta => "magenta",
            Color::Cyan => "cyan",
            Color::White => "white",
            Color::Black => "black",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors_to_string() {
        assert_eq!(Color::Red.to_string(), "red");
        assert_eq!(Color::Blue.to_string(), "blue");
        assert_eq!(Color::Green.to_string(), "green");
        assert_eq!(Color::Yellow.to_string(), "yellow");
        assert_eq!(Color::Magenta.to_string(), "magenta");
        assert_eq!(Color::Cyan.to_string(), "cyan");
        assert_eq!(Color::White.to_string(), "white");
        assert_eq!(Color::Black.to_string(), "black");
    }
}