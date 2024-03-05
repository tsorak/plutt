pub mod pos {
    use crossterm::{cursor::MoveTo, terminal::size};

    pub fn bl() -> MoveTo {
        let (_cols, rows) = size().expect("Could not get terminal size");
        MoveTo(0, rows)
    }

    pub fn br(inset: u16) -> MoveTo {
        let (cols, rows) = size().expect("Could not get terminal size");
        MoveTo(cols - inset, rows)
    }
}
