use std::{net::UdpSocket, thread};

use log::{debug, error, info};

pub struct PongServer {
    socket: Option<UdpSocket>,
}

impl PongServer {
    pub fn new() -> PongServer {
        Self { socket: None }
    }

    pub fn serve(&mut self) -> std::io::Result<()> {
        let server_address = "127.0.0.1:8080";
        let socket = UdpSocket::bind(server_address)?;
        info!("Server listening at {server_address}");

        let recv_socket = socket.try_clone()?;
        self.socket = Some(socket);

        // receiver thread
        let recv_handle = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                match recv_socket.recv_from(&mut buf) {
                    Ok((size, client_address)) => {
                        let received_bytes = &buf[..size];
                        let str_message = String::from_utf8_lossy(received_bytes);
                        debug!(
                            "received: {received_bytes:?}  = {str_message} from {client_address}"
                        );
                        if str_message == "ping" {
                            // Respond to ping right away
                            let msg = b"pong";
                            recv_socket
                                .send_to(msg, client_address)
                                .expect("Could not reply to client");
                            debug!("Replied to client: {}", String::from_utf8_lossy(msg));
                        }
                    }
                    Err(e) => error!("recv error: {e}"),
                }
            }
        });

        recv_handle.join().unwrap();

        Ok(())
    }
}
