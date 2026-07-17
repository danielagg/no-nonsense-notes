use super::*;

mod connected;

use connected::{configure_timeout, connected_loop};

pub(super) fn run(
    url: String,
    token: String,
    store: Arc<NativeStore>,
    delegate: Arc<dyn SyncDelegate>,
    commands: mpsc::Receiver<Command>,
) {
    let mut retry = 0u32;
    loop {
        if matches!(commands.try_recv(), Ok(Command::Stop)) {
            break;
        }
        delegate.state_changed(SyncState::Connecting, None);
        match connect(url.as_str()) {
            Ok((mut socket, _)) => {
                configure_timeout(&mut socket);
                if socket.send(Message::Text(token.clone().into())).is_err() {
                    delegate
                        .state_changed(SyncState::Error, Some("authentication send failed".into()));
                } else if connected_loop(&mut socket, &store, delegate.as_ref(), &commands) {
                    break;
                }
            }
            Err(error) => delegate.state_changed(SyncState::Error, Some(error.to_string())),
        }
        delegate.state_changed(SyncState::Disconnected, None);
        retry = retry.saturating_add(1);
        let delay = Duration::from_secs(2u64.saturating_pow(retry.min(3)));
        if matches!(commands.recv_timeout(delay), Ok(Command::Stop)) {
            break;
        }
    }
    delegate.state_changed(SyncState::Disconnected, None);
}
