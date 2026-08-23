pub mod auth;
pub mod error;
pub mod handlers;

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use snapdown_store::sqlite::{SqliteAccessKeyStore, SqliteBundleStore};
use snapdown_store::vault::VaultBlobStore;
use tiny_http::Server;

pub struct LocalApiServer {
    addr: SocketAddr,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl LocalApiServer {
    pub fn start(
        port: u16,
        key_store: Arc<SqliteAccessKeyStore>,
        bundle_store: Arc<SqliteBundleStore>,
        vault_store: Arc<VaultBlobStore>,
    ) -> Result<Self, String> {
        let bind_addr = format!("127.0.0.1:{port}");
        let server = Server::http(&bind_addr).map_err(|e| e.to_string())?;
        let addr = server
            .server_addr()
            .to_ip()
            .ok_or_else(|| "Failed to get server IP address".to_string())?;

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let server_arc = Arc::new(server);
        let server_inner = server_arc.clone();

        let handle = thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                match server_inner.recv_timeout(std::time::Duration::from_millis(200)) {
                    Ok(Some(req)) => {
                        handlers::handle_http_request(
                            req,
                            key_store.clone(),
                            bundle_store.clone(),
                            vault_store.clone(),
                        );
                    }
                    Ok(None) => {}
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            addr,
            running,
            handle: Some(handle),
        })
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for LocalApiServer {
    fn drop(&mut self) {
        self.stop();
    }
}
