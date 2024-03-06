pub mod terminal {
    use std::io::stdout;

    use crossterm::execute;

    pub fn clear_all() {
        let _ = execute!(
            stdout(),
            crossterm::cursor::SavePosition,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::RestorePosition
        );
    }
}

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
