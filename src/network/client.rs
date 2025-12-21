use std::{
    error::Error,
    net::UdpSocket,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use log::{debug, error, info};

use crate::scenes::metrics::SimpleMovingAverage;

pub struct GameClient {
    healthy: Arc<RwLock<bool>>,
    ping_sma: Arc<RwLock<SimpleMovingAverage>>,
    //last_ping_sent: Arc<Mutex<Instant>>,
}

impl GameClient {
    pub fn new(server_address: &str) -> Result<GameClient, Box<dyn Error>> {
        // Bind to 0.0.0.0:0 to let OS pick an available port
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(server_address)?;
        info!(
            "Initialized client at {}, connected to {server_address}",
            socket.local_addr()?
        );

        let sma = Arc::new(RwLock::new(SimpleMovingAverage::new(5)));
        let sma_ping_thread = Arc::clone(&sma);
        let health_lock = Arc::new(RwLock::new(false));
        let health_lock_pthread = Arc::clone(&health_lock);

        // Spawn communication thread
        thread::spawn(move || {
            let msg = b"ping";
            loop {
                let start = Instant::now();
                if let Err(e) = socket.send(msg) {
                    error!("send error: {e}");
                    *health_lock_pthread.write().unwrap() = false;
                }

                let mut buf = [0u8; 1024];
                socket
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                match socket.recv(&mut buf) {
                    Ok(n) => {
                        let duration = start.elapsed();
                        debug!(
                            "Ping reply received ({} bytes) RTT: {} ms",
                            n,
                            duration.as_millis()
                        );
                        // TODO: Update healthy
                        sma_ping_thread.write().unwrap().add_elapsed(start);
                        *health_lock_pthread.write().unwrap() = true;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        error!("Ping timeout");
                        *health_lock_pthread.write().unwrap() = false;
                    }
                    Err(e) => {
                        error!("recv error: {e}");
                        *health_lock_pthread.write().unwrap() = false;
                    }
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        Ok(GameClient {
            healthy: health_lock,
            ping_sma: sma,
        })
    }

    pub fn render_ui(&self, ui: &mut imgui::Ui) {
        ui.window("Network")
            .size([150.0, 100.0], imgui::Condition::FirstUseEver)
            .position([500.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Health: {}", self.healthy.read().unwrap()));
                ui.text(format!(
                    "Ping: {:.1}ns",
                    self.ping_sma.read().unwrap().get()
                ));
            });
    }
}
