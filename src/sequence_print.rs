use std::{fmt::Display, io::stdout};

use tokio::{
    sync::broadcast::Receiver,
    task::{JoinError, JoinHandle},
};

use crate::{
    ext::crossterm as ct,
    input::{vim_key::VimKey, vim_sequence::VimSequence},
};

pub struct SequencePrinter {
    handle: Option<JoinHandle<()>>,
}

impl SequencePrinter {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn start(&mut self, input: Receiver<VimKey>) -> &mut Self {
        if self.handle.is_some() {
            return self;
        }

        let h = tokio::spawn(async move {
            let mut vim_sequence = VimSequence::new();
            vim_sequence
                .setup_sequence_channel()
                .attach_input_consumer(input);

            loop {
                if let Some(seq) = vim_sequence.recv().await {
                    match seq.as_ref() {
                        "q" => break,
                        _seq => Self::print(&seq),
                    }
                }
            }
        });

        self.handle = Some(h);
        self
    }

    pub async fn wait_end(mut self) -> Result<(), JoinError> {
        if let Some(h) = self.handle.take() {
            h.await?
        }
        Ok(())
    }

    pub fn print<T>(v: &T)
    where
        T: Display,
    {
        let _ = crossterm::execute!(
            stdout(),
            crossterm::cursor::SavePosition,
            ct::pos::br(v.to_string().len() as u16),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            crossterm::style::Print(v),
            crossterm::cursor::RestorePosition
        )
        .inspect_err(|e| {
            dbg!(e);
        });
    }
}
