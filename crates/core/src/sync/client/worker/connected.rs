use super::super::*;

pub(super) fn connected_loop(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    store: &NativeStore,
    delegate: &dyn SyncDelegate,
    commands: &mpsc::Receiver<Command>,
) -> bool {
    let mut ready = false;
    let mut pull_pending = true;
    let mut in_flight: Option<PendingPush> = None;
    loop {
        match commands.try_recv() {
            Ok(Command::Stop) => {
                let _ = socket.close(None);
                return true;
            }
            Ok(Command::Wake) => pull_pending = true,
            Err(mpsc::TryRecvError::Disconnected) => return true,
            Err(mpsc::TryRecvError::Empty) => {}
        }

        if ready && in_flight.is_none() {
            match store.next_pending() {
                Ok(Some(pending)) => match store.frame_for(&pending) {
                    Ok(frame) => {
                        if socket.send(Message::Binary(frame.into())).is_err() {
                            return false;
                        }
                        in_flight = Some(pending);
                    }
                    Err(error) => delegate.state_changed(SyncState::Error, Some(error.to_string())),
                },
                Ok(None) if pull_pending => {
                    let cursor = store.cursor().unwrap_or(0);
                    if socket
                        .send(Message::Text(protocol::encode_pull_request(cursor).into()))
                        .is_err()
                    {
                        return false;
                    }
                    pull_pending = false;
                }
                Ok(None) => {}
                Err(error) => delegate.state_changed(SyncState::Error, Some(error.to_string())),
            }
        }

        match socket.read() {
            Ok(Message::Text(text)) if text == "ready" => {
                ready = true;
                pull_pending = true;
                delegate.state_changed(SyncState::Connected, None);
            }
            Ok(Message::Text(text)) if text == "unauthorized" => {
                delegate.state_changed(SyncState::Error, Some("unauthorized".into()));
                return true;
            }
            Ok(Message::Text(text)) if text.starts_with("update:") => pull_pending = true,
            Ok(Message::Text(text)) if text.starts_with("seq:") => {
                match store.apply_pull_response(&text) {
                    Ok(count) => {
                        if count > 0 {
                            delegate.notes_changed();
                        }
                    }
                    Err(error) => delegate.state_changed(SyncState::Error, Some(error.to_string())),
                }
            }
            Ok(Message::Binary(bytes)) => {
                if protocol::decode_push_response(&bytes).is_ok() {
                    if let Some(pending) = in_flight.take() {
                        let _ = store.acknowledge(&pending);
                    }
                    pull_pending = true;
                }
            }
            Ok(Message::Close(_)) => return false,
            Ok(Message::Ping(data)) => {
                let _ = socket.send(Message::Pong(data));
            }
            Ok(_) => {}
            Err(WsError::Io(error))
                if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(WsError::ConnectionClosed | WsError::AlreadyClosed) => return false,
            Err(error) => {
                delegate.state_changed(SyncState::Error, Some(error.to_string()));
                return false;
            }
        }
    }
}

pub(super) fn configure_timeout(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
    let timeout = Some(Duration::from_millis(200));
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => {
            let _ = stream.set_read_timeout(timeout);
        }
        MaybeTlsStream::Rustls(stream) => {
            let _ = stream.sock.set_read_timeout(timeout);
        }
        _ => {}
    }
}
