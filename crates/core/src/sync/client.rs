use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Error as WsError, Message, WebSocket, connect};

use crate::storage::native::{NativeStore, PendingPush};
use crate::sync::protocol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

pub trait SyncDelegate: Send + Sync + 'static {
    fn state_changed(&self, state: SyncState, detail: Option<String>);
    fn notes_changed(&self);
}

enum Command {
    Wake,
    Stop,
}

pub struct SyncClient {
    commands: mpsc::Sender<Command>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl SyncClient {
    pub fn start(
        url: String,
        token: String,
        store: Arc<NativeStore>,
        delegate: Arc<dyn SyncDelegate>,
    ) -> Self {
        let (commands, receiver) = mpsc::channel();
        let thread = thread::spawn(move || worker::run(url, token, store, delegate, receiver));
        Self {
            commands,
            thread: Mutex::new(Some(thread)),
        }
    }

    pub fn wake(&self) {
        let _ = self.commands.send(Command::Wake);
    }

    pub fn stop(&self) {
        let _ = self.commands.send(Command::Stop);
        if let Ok(mut guard) = self.thread.lock() {
            if let Some(thread) = guard.take() {
                let _ = thread.join();
            }
        }
    }
}

impl Drop for SyncClient {
    fn drop(&mut self) {
        self.stop();
    }
}

mod worker;
