use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

pub struct PingResult {
    pub latency: Option<Duration>,
    pub error: Option<String>,
}

pub fn ping(address: &str, port: u16, timeout: Duration) -> PingResult {
    let target = format!("{address}:{port}");
    let start = Instant::now();

    let socket_addresses = match target.to_socket_addrs() {
        Ok(addresses) => addresses,
        Err(error) => {
            return PingResult {
                latency: None,
                error: Some(format!("DNS resolution failed: {error}")),
            };
        }
    };

    let mut last_error = None;

    for socket_address in socket_addresses {
        match TcpStream::connect_timeout(&socket_address, timeout) {
            Ok(_) => {
                return PingResult {
                    latency: Some(start.elapsed()),
                    error: None,
                };
            }
            Err(error) => {
                last_error = Some(error.to_string());
            }
        }
    }

    PingResult {
        latency: None,
        error: Some(last_error.unwrap_or_else(|| "No addresses available".to_string())),
    }
}
