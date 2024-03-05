use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
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

fn is_ctrl_c(k: KeyEvent) -> bool {
    k.code == KeyCode::Char('c') && k.modifiers.iter().any(|m| m == KeyModifiers::CONTROL)
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

    pub fn init(&mut self) -> &mut Self {
        let tx = self
            .tx
            .take()
            .expect("Producer already taken. 'init' can not run more than once.");

        tokio::spawn(async move {
            let mut input = crossterm::event::EventStream::new();
            loop {
                match input.next().fuse().await {
                    Some(Ok(Event::Key(key))) if is_ctrl_c(key) => {
                        crossterm::terminal::disable_raw_mode();
                        std::process::exit(0);
                    }
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
    use tokio::sync::{
        broadcast::{channel, Receiver, Sender},
        Mutex,
    };

    pub struct VimSequence {
        buffer: Arc<Mutex<Vec<char>>>,
        sequence_tx: Option<Sender<String>>,
        sequence_rx: Option<Receiver<String>>,
    }

    impl VimSequence {
        pub fn new() -> Self {
            Self {
                buffer: Arc::new(Mutex::new(vec![])),
                sequence_tx: None,
                sequence_rx: None,
            }
        }

        pub fn setup_sequence_channel(&mut self) -> &mut Self {
            let (tx, rx) = channel::<String>(8);
            self.sequence_tx.get_or_insert(tx);
            self.sequence_rx.get_or_insert(rx);

            self
        }

        pub fn attach_input_consumer(&mut self, mut input_rx: Receiver<char>) -> &Self {
            let buffer = self.buffer.clone();
            let sequence_tx = self.sequence_tx.take();

            tokio::spawn(async move {
                loop {
                    if let Ok(char) = input_rx.recv().await {
                        let mut lock = buffer.lock().await;
                        (*lock).push(char);

                        if sequence_tx.is_none() {
                            continue;
                        };

                        let tx = sequence_tx.as_ref().unwrap().clone();
                        let buf = (*lock).clone();
                        Self::broadcast_buffer_update(tx, buf);
                    }
                }
            });

            self
        }

        fn broadcast_buffer_update(tx: Sender<String>, seq: Vec<char>) {
            tokio::spawn(async move { tx.clone().send(chars_to_string(&seq)) });
        }

        pub async fn recv(&mut self) -> Option<String> {
            let mut rx = self
                .sequence_rx
                .as_ref()
                .expect("Channel should be set up")
                .resubscribe();

            match rx.recv().await {
                Ok(vim_sequence) => Some(vim_sequence),
                Err(err) => {
                    dbg!(err);
                    None
                }
            }
        }

        pub async fn to_string(&self) -> String {
            let lock = self.buffer.lock().await;
            chars_to_string(&lock)
        }
    }

    fn chars_to_string(chars: &[char]) -> String {
        let mut s = String::new();
        chars.iter().for_each(|char| s.push(*char));
        s
    }
}
