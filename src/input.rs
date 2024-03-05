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

pub mod vim_sequence {
    use std::sync::Arc;
    use tokio::sync::{broadcast::Receiver, Mutex};

    pub struct VimSequence {
        buffer: Arc<Mutex<Vec<char>>>,
    }

    impl VimSequence {
        pub fn new(input_rx: Receiver<char>) -> Self {
            let buffer = Arc::new(Mutex::new(vec![]));

            Self::attach_input_consumer(&buffer, input_rx);

            Self { buffer }
        }

        pub async fn to_string(&self) -> String {
            let mut s = String::new();
            let lock = self.buffer.lock().await;
            lock.iter().for_each(|char| s.push(*char));
            s
        }

        fn attach_input_consumer(buffer: &Arc<Mutex<Vec<char>>>, mut input_rx: Receiver<char>) {
            let buffer = buffer.clone();

            tokio::spawn(async move {
                loop {
                    if let Ok(char) = input_rx.recv().await {
                        buffer.lock().await.push(char);
                    }
                }
            });
        }
    }
}
