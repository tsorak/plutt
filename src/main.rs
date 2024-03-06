mod edit_ui;
mod ext;
mod input;
mod sequence_print;

use crate::{input::Input, sequence_print::SequencePrinter};

#[tokio::main]
async fn main() {
    let _ = crossterm::terminal::enable_raw_mode();
    ext::crossterm::terminal::clear_all();

    let mut input = Input::new();
    input.init();

    let mut seq_printer = SequencePrinter::new();
    seq_printer.start(input.get_receiver());

    let _ = seq_printer.wait_end().await;

    let _ = crossterm::terminal::disable_raw_mode();

    // let sequence_printer_handle = SequencePrinter::new(input.get_receiver());
    // let editor_handle = Editor::new(input.get_receiver());
}
