use crossterm::event::{Event, KeyCode, KeyEvent};
use futures::{future::FutureExt, StreamExt};

use tokio::{
    sync::broadcast::{channel, Receiver, Sender},
    task::JoinHandle,
};

pub struct Input {
    tx: Option<Sender<char>>,
    rx: Receiver<char>,
    stdin_reader_handle: Option<JoinHandle<()>>,
}

impl Input {
    pub fn new() -> Self {
        let (tx, rx) = channel::<char>(8);

        Self {
            tx: Some(tx),
            rx,
            stdin_reader_handle: None,
        }
    }

    pub fn get_receiver(&self) -> Receiver<char> {
        self.rx.resubscribe()
    }

    fn init(&mut self) -> &mut Self {
        let tx = self
            .tx
            .take()
            .expect("Producer already taken. 'init' can not run more than once.");

        tokio::spawn(async move {
            let mut input = crossterm::event::EventStream::new();
            loop {
                match input.next().fuse().await {
                    Some(Ok(Event::Key(key))) => Self::handle_key(&tx, key),
                    _ => (),
                }
            }
        });

        self
    }

    fn handle_key(tx: &Sender<char>, key: KeyEvent) {
        if let KeyCode::Char(c) = key.code {
            tx.send(c);
        }
    }
}
