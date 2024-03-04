mod edit_ui;
mod input;
mod sequence_print;

use crate::input::Input;

#[tokio::main]
async fn main() {
    let input = Input::new();

    // let sequence_print_handle = sequence_print::new(input.get_receiver());
    // let editor_handle = sequence_print::new(input.get_receiver());
    println!("Hello, world!");
}
