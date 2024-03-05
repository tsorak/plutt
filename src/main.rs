#![allow(dead_code)]
mod edit_ui;
mod input;
mod sequence_print;

use crate::input::Input;

#[tokio::main]
async fn main() {
    let input = Input::new();

    // let sequence_printer_handle = SequencePrinter::new(input.get_receiver());
    // let editor_handle = Editor::new(input.get_receiver());
    println!("Hello, world!");
}
