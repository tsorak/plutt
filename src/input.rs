use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use futures::{future::FutureExt, StreamExt};

use tokio::{
    sync::broadcast::{channel, Receiver, Sender},
    task::JoinHandle,
};

use self::vim_key::VimKey;

pub struct Input {
    tx: Option<Sender<VimKey>>,
    rx: Receiver<VimKey>,
    stdin_reader_handle: Option<JoinHandle<()>>,
}

fn is_ctrl_c(k: KeyEvent) -> bool {
    k.code == KeyCode::Char('c') && k.modifiers.iter().any(|m| m == KeyModifiers::CONTROL)
}

impl Input {
    pub fn new() -> Self {
        let (tx, rx) = channel::<VimKey>(8);

        Self {
            tx: Some(tx),
            rx,
            stdin_reader_handle: None,
        }
    }

    pub fn get_receiver(&self) -> Receiver<VimKey> {
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
                        let _ = crossterm::terminal::disable_raw_mode();
                        std::process::exit(0);
                    }
                    Some(Ok(Event::Key(KeyEvent { code, .. }))) => Self::handle_keycode(&tx, code),
                    _ => (),
                }
            }
        });

        self
    }

    fn handle_keycode(tx: &Sender<VimKey>, key: KeyCode) {
        if let Ok(vim_key) = VimKey::try_from(key) {
            let _ = tx.send(vim_key);
        }
    }
}

pub mod vim_key {
    use crossterm::event::KeyCode;

    #[derive(Debug, Clone)]
    pub struct VimKey {
        pub alphanumeric: Option<char>,
        pub special: Option<String>,
    }

    impl VimKey {
        fn alphanumeric_key(c: char) -> Self {
            Self {
                alphanumeric: Some(c),
                special: None,
            }
        }

        fn special_key(key: &str) -> Self {
            Self {
                alphanumeric: None,
                special: Some(format!("<{key}>")),
            }
        }
    }

    impl TryFrom<KeyCode> for VimKey {
        fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
            match value {
                KeyCode::Char(c) => Ok(Self::alphanumeric_key(c)),
                KeyCode::Esc => Ok(Self::special_key("esc")),
                KeyCode::Tab => Ok(Self::special_key("tab")),
                KeyCode::Backspace => Ok(Self::special_key("backspace")),
                _ => Err(()),
            }
        }

        type Error = ();
    }
}

pub mod vim_sequence {
    use std::sync::Arc;
    use tokio::sync::{
        broadcast::{channel, Receiver, Sender},
        Mutex,
    };

    use super::vim_key::VimKey;

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

        pub fn attach_input_consumer(&mut self, input_rx: Receiver<VimKey>) -> &Self {
            let buffer = self.buffer.clone();
            let sequence_tx = self.sequence_tx.take();
            // let special_handlers = self.special_key_handlers.as_ref();

            // if let Some(special_handlers) = special_handlers {
            Self::start_receiving(buffer, input_rx, sequence_tx);
            // } else {
            // Self::start_receiving_chars_only(buffer, input_rx, sequence_tx);
            // }

            self
        }

        fn broadcast_buffer_update(tx: Sender<String>, seq: Vec<char>) {
            tokio::spawn(async move { tx.send(chars_to_string(&seq)) });
        }

        fn start_receiving(
            buffer: Arc<Mutex<Vec<char>>>,
            mut input_rx: Receiver<VimKey>,
            sequence_tx: Option<Sender<String>>,
            // _special_handlers: Arc<HashMap<String, ()>>,
        ) {
            tokio::spawn(async move {
                loop {
                    match input_rx.recv().await {
                        Ok(VimKey {
                            special: Some(key), ..
                        }) => {
                            Self::handle_special_key(key, buffer.clone(), sequence_tx.clone())
                                .await;
                        }
                        Ok(VimKey {
                            alphanumeric: Some(key),
                            ..
                        }) => {
                            let mut lock = buffer.lock().await;
                            (*lock).push(key);

                            if let Some(ref tx) = sequence_tx {
                                Self::broadcast_buffer_update(tx.clone(), (*lock).clone());
                            };
                        }
                        _ => (),
                    }
                }
            });
        }

        fn start_receiving_chars_only(
            buffer: Arc<Mutex<Vec<char>>>,
            mut input_rx: Receiver<VimKey>,
            sequence_tx: Option<Sender<String>>,
        ) {
            tokio::spawn(async move {
                loop {
                    if let Ok(VimKey {
                        alphanumeric: Some(key),
                        ..
                    }) = input_rx.recv().await
                    {
                        let mut lock = buffer.lock().await;
                        (*lock).push(key);

                        if let Some(ref tx) = sequence_tx {
                            Self::broadcast_buffer_update(tx.clone(), (*lock).clone());
                        };
                    }
                }
            });
        }

        async fn handle_special_key(
            key: String,
            buffer: Arc<Mutex<Vec<char>>>,
            sequence_tx: Option<Sender<String>>,
        ) {
            match key.as_ref() {
                "<esc>" => {
                    Self::buffer_clear(buffer).await;

                    if let Some(ref tx) = sequence_tx {
                        let _ = tx.send("".into());
                    }
                }
                _ => (),
            }
        }

        async fn buffer_clear(buffer: Arc<Mutex<Vec<char>>>) {
            (*buffer.lock().await).clear()
        }

        async fn buffer_len(buffer: Arc<Mutex<Vec<char>>>) -> usize {
            (*buffer.lock().await).len()
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
