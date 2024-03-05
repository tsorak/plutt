mod edit_ui;
mod input;
mod sequence_print;

use std::io::stdout;

use crate::input::{vim_sequence::VimSequence, Input};

#[tokio::main]
async fn main() {
    let mut input = Input::new();
    input.init();

    let mut vim_sequence = VimSequence::new();
    vim_sequence
        .setup_sequence_channel()
        .attach_input_consumer(input.get_receiver());

    crossterm::terminal::enable_raw_mode();

    loop {
        if let Some(seq) = vim_sequence.recv().await {
            match seq.as_ref() {
                "q" => break,
                sequence => {
                    let s = sequence.to_string();
                    crossterm::execute!(
                        stdout(),
                        crossterm::cursor::MoveTo(0, 0),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                        crossterm::style::Print(&s)
                    );
                }
            }
        }
    }

    crossterm::terminal::disable_raw_mode();

    // let sequence_printer_handle = SequencePrinter::new(input.get_receiver());
    // let editor_handle = Editor::new(input.get_receiver());
}
