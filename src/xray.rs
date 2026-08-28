use std::net::ToSocketAddrs;
use std::process::{Child, Command, Stdio};

pub fn resolve_proxy_address(address: &str) -> Result<String, String> {
    if address.parse::<std::net::IpAddr>().is_ok() {
        return Ok(address.to_string());
    }

    let socket_address = format!("{}:443", address);

    let mut addresses = socket_address
        .to_socket_addrs()
        .map_err(|error| format!("Failed to resolve {}: {}", address, error))?;

    addresses
        .next()
        .map(|addr| addr.ip().to_string())
        .ok_or_else(|| format!("Could not resolve {}", address))
}

pub fn configure_tun(proxy_ip: &str) -> Result<(), String> {
    println!();
    println!("Configuring TUN...");
    println!("────────────────────────");
    println!("Proxy server: {}", proxy_ip);

    let route_get = Command::new("ip")
        .args(["route", "get", proxy_ip])
        .output()
        .map_err(|error| format!("Failed to query route: {}", error))?;

    if !route_get.status.success() {
        return Err(format!(
            "Failed to determine route to {}: {}",
            proxy_ip,
            String::from_utf8_lossy(&route_get.stderr)
        ));
    }

    let route_output = String::from_utf8_lossy(&route_get.stdout);

    let parts: Vec<&str> = route_output.split_whitespace().collect();

    let mut gateway = None;
    let mut interface = None;

    for i in 0..parts.len() {
        if parts[i] == "via" && i + 1 < parts.len() {
            gateway = Some(parts[i + 1]);
        }

        if parts[i] == "dev" && i + 1 < parts.len() {
            interface = Some(parts[i + 1]);
        }
    }

    let interface = interface.ok_or_else(|| "Could not determine network interface".to_string())?;

    println!("Uplink interface: {}", interface);

    if let Some(gateway) = gateway {
        println!("Uplink gateway: {}", gateway);

        let status = Command::new("sudo")
            .args([
                "ip",
                "route",
                "replace",
                &format!("{}/32", proxy_ip),
                "via",
                gateway,
                "dev",
                interface,
            ])
            .status()
            .map_err(|error| format!("Failed to create proxy route: {}", error))?;

        if !status.success() {
            return Err("Failed to create route to Xray server".to_string());
        }
    } else {
        let status = Command::new("sudo")
            .args([
                "ip",
                "route",
                "replace",
                &format!("{}/32", proxy_ip),
                "dev",
                interface,
            ])
            .status()
            .map_err(|error| format!("Failed to create proxy route: {}", error))?;

        if !status.success() {
            return Err("Failed to create route to Xray server".to_string());
        }
    }

    println!("Proxy route configured.");

    let status = Command::new("sudo")
        .args(["ip", "addr", "replace", "10.10.0.1/30", "dev", "xray0"])
        .status()
        .map_err(|error| format!("Failed to configure xray0 address: {}", error))?;

    if !status.success() {
        return Err("Failed to assign address to xray0".to_string());
    }

    println!("xray0: 10.10.0.1/30");

    let status = Command::new("sudo")
        .args(["ip", "route", "replace", "default", "dev", "xray0"])
        .status()
        .map_err(|error| format!("Failed to configure default route: {}", error))?;

    if !status.success() {
        return Err("Failed to configure default route through xray0".to_string());
    }

    println!("Default route: xray0");

    Ok(())
}

pub fn prepare_xray_config(mut config: serde_json::Value) -> Result<serde_json::Value, String> {
    let inbounds = config
        .get_mut("inbounds")
        .and_then(|value| value.as_array_mut())
        .ok_or_else(|| "inbounds not found".to_string())?;

    let mut socks_found = false;

    for inbound in inbounds.iter_mut() {
        let is_socks = inbound.get("protocol").and_then(|value| value.as_str()) == Some("socks");

        if is_socks {
            inbound["listen"] = serde_json::Value::String("127.0.0.1".to_string());
            inbound["port"] = serde_json::Value::Number(9999.into());

            socks_found = true;
        }
    }

    if !socks_found {
        return Err("SOCKS inbound not found".to_string());
    }

    let tun_exists = inbounds
        .iter()
        .any(|inbound| inbound.get("protocol").and_then(|value| value.as_str()) == Some("tun"));

    if !tun_exists {
        let tun = serde_json::json!({
            "tag": "tun",
            "protocol": "tun",
            "settings": {
                "name": "xray0",
                "mtu": 1500,
                "gateway": [
                    "10.10.0.1/30"
                ],
                "autoSystemRoutingTable": true,
                "autoOutboundsInterface": "auto"
            },
            "sniffing": {
                "enabled": true,
                "destOverride": [
                    "http",
                    "tls",
                    "quic"
                ],
                "routeOnly": true
            }
        });

        inbounds.push(tun);
    }

    Ok(config)
}

pub fn is_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

use std::fs::OpenOptions;
use std::time::Duration;

pub fn start(config_file: &std::path::Path, log_file: &std::path::Path) -> Result<Child, String> {
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .map_err(|error| format!("Failed to open log: {error}"))?;

    let log_error = log
        .try_clone()
        .map_err(|error| format!("Failed to clone log: {error}"))?;

    Command::new("xray")
        .arg("run")
        .arg("-config")
        .arg(config_file)
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_error))
        .spawn()
        .map_err(|error| format!("Failed to start Xray: {error}"))
}

pub fn wait_for_tun(name: &str) -> Result<(), String> {
    for _ in 0..50 {
        let status = Command::new("ip")
            .args(["link", "show", name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if matches!(status, Ok(status) if status.success()) {
            return Ok(());
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Err(format!("Interface {} did not appear", name))
}

pub fn stop(pid: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .arg(pid.to_string())
        .status()
        .map_err(|error| format!("Failed to execute kill: {error}"))?;

    if !status.success() {
        return Err(format!("Failed to stop Xray (PID {})", pid));
    }

    Ok(())
}
