use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::net::ToSocketAddrs;
use std::process::{Command, Stdio};
mod ui;

#[derive(Serialize, Deserialize)]
struct Config {
    subscription_url: String,
    hwid: String,
}

fn get_proxy_address(config: &serde_json::Value) -> Result<String, String> {
    let proxy = config
        .get("outbounds")
        .and_then(|value| value.as_array())
        .and_then(|outbounds| {
            outbounds.iter().find(|outbound| {
                outbound.get("tag").and_then(|value| value.as_str()) == Some("proxy")
            })
        })
        .ok_or_else(|| "Proxy outbound not found".to_string())?;

    let settings = proxy
        .get("settings")
        .ok_or_else(|| "Proxy settings not found".to_string())?;

    // Hysteria
    if let Some(address) = settings.get("address").and_then(|value| value.as_str()) {
        return Ok(address.to_string());
    }

    // VLESS
    if let Some(address) = settings
        .get("vnext")
        .and_then(|value| value.as_array())
        .and_then(|vnext| vnext.first())
        .and_then(|server| server.get("address"))
        .and_then(|value| value.as_str())
    {
        return Ok(address.to_string());
    }

    Err("Proxy address not found".to_string())
}

fn resolve_proxy_address(address: &str) -> Result<String, String> {
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

fn configure_tun(proxy_ip: &str) -> Result<(), String> {
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

fn prepare_xray_config(mut config: serde_json::Value) -> Result<serde_json::Value, String> {
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
                "MTU": 1500
            }
        });

        inbounds.push(tun);
    }

    Ok(config)
}

#[derive(Parser)]
#[command(
    name = "xrayctl",
    version,
    about = "Xray subscription manager",
    long_about = "A simple command-line manager for Xray subscriptions and connections."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(
        name = "sub",
        alias = "subscription",
        about = "Manage subscriptions and Xray"
    )]
    Subscription(SubArgs),
}

#[derive(clap::Args)]
struct SubArgs {
    /// Start Xray
    #[arg(short = 's', long = "start", conflicts_with_all = ["stop", "status", "list", "update", "generate"])]
    start: bool,

    /// Stop Xray
    #[arg(short = 'x', long = "stop", conflicts_with_all = ["start", "status", "list", "update", "generate"])]
    stop: bool,

    /// Show Xray status
    #[arg(short = 't', long = "status", conflicts_with_all = ["start", "stop", "list", "update", "generate"])]
    status: bool,

    /// List profiles
    #[arg(short = 'l', long = "list", conflicts_with_all = ["start", "stop", "status", "update", "generate"])]
    list: bool,

    /// Update subscription
    #[arg(short = 'u', long = "update", conflicts_with_all = ["start", "stop", "status", "list", "generate"])]
    update: bool,

    /// Generate Xray configuration
    #[arg(short = 'g', long = "generate", conflicts_with_all = ["start", "stop", "status", "list", "update"])]
    generate: bool,

    #[command(subcommand)]
    command: Option<SubscriptionCommand>,
}

#[derive(Subcommand)]
enum SubscriptionCommand {
    #[command(about = "Add a subscription")]
    Add { url: String },

    #[command(about = "Show subscription URL")]
    Show,

    #[command(about = "Download subscription")]
    Fetch,

    #[command(about = "Show subscription information")]
    Info,

    #[command(about = "List available profiles")]
    List,

    #[command(about = "Debug subscription profiles")]
    Debug,

    #[command(about = "Show detailed profile information")]
    ShowProfile { index: usize },

    #[command(about = "Select active profile")]
    Use { index: usize },

    #[command(about = "Generate Xray configuration")]
    Generate,

    #[command(about = "Start Xray")]
    Start { index: Option<usize> },

    #[command(about = "Stop Xray")]
    Stop,

    #[command(about = "Show Xray status")]
    Status,

    #[command(about = "Update subscription")]
    Update,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Subscription(args) => {
            let command = if args.start {
                Some(SubscriptionCommand::Start { index: None })
            } else if args.stop {
                Some(SubscriptionCommand::Stop)
            } else if args.status {
                Some(SubscriptionCommand::Status)
            } else if args.list {
                Some(SubscriptionCommand::List)
            } else if args.update {
                Some(SubscriptionCommand::Update)
            } else if args.generate {
                Some(SubscriptionCommand::Generate)
            } else {
                args.command
            };

            match command {
                Some(SubscriptionCommand::Add { url }) => {
                    let config = Config {
                        subscription_url: url,
                        hwid: String::from("xrayctl-test-hwid"),
                    };

                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");

                    std::fs::create_dir_all(&xrayctl_dir).unwrap();

                    let config_file = xrayctl_dir.join("config.toml");

                    let toml = toml::to_string(&config).unwrap();

                    std::fs::write(&config_file, &toml).unwrap();

                    println!("{}", config.subscription_url);
                }
                Some(SubscriptionCommand::Show) => {
                    let config_dir = dirs::config_dir().unwrap();

                    let xrayctl_dir = config_dir.join("xrayctl");

                    let config_file = xrayctl_dir.join("config.toml");

                    let toml = std::fs::read_to_string(&config_file).unwrap();

                    let config: Config = toml::from_str(&toml).unwrap();

                    println!("{}", config.subscription_url);
                }
                Some(SubscriptionCommand::Fetch) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");
                    let config_file = xrayctl_dir.join("config.toml");

                    let toml = std::fs::read_to_string(&config_file).unwrap();

                    let config: Config = toml::from_str(&toml).unwrap();

                    let client = reqwest::blocking::Client::new();

                    let response = client
                        .get(&config.subscription_url)
                        .header("User-Agent", "Happ/3.13.0")
                        .header("X-HWID", &config.hwid)
                        .header("X-Device-Os", "Android")
                        .header("X-Device-Locale", "ru")
                        .header("X-Device-Model", "xrayctl")
                        .header("X-Ver-Os", "17")
                        .send()
                        .unwrap();

                    println!("Status: {}", response.status());

                    for (name, value) in response.headers() {
                        println!("{name}: {value:?}");
                    }

                    let body = response.text().unwrap();

                    println!("Downloaded: {} bytes", body.len());

                    let subscription: serde_json::Value = match serde_json::from_str(&body) {
                        Ok(json) => json,
                        Err(error) => {
                            println!("Invalid subscription JSON: {error}");
                            return;
                        }
                    };

                    println!("Valid subscription JSON!");

                    let subscription_file = xrayctl_dir.join("subscription.json");

                    let formatted = serde_json::to_string_pretty(&subscription).unwrap();

                    std::fs::write(&subscription_file, formatted).unwrap();

                    println!("Subscription saved:");
                    println!("{}", subscription_file.display());
                }
                Some(SubscriptionCommand::Info) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");
                    let config_file = xrayctl_dir.join("config.toml");

                    let toml = std::fs::read_to_string(&config_file).unwrap();

                    let config: Config = toml::from_str(&toml).unwrap();

                    let client = reqwest::blocking::Client::new();

                    let response = client
                        .get(&config.subscription_url)
                        .header("User-Agent", "Happ/3.13.0")
                        .header("X-Device-Os", "Android")
                        .header("X-Device-Locale", "ru")
                        .header("X-Device-Model", "xrayctl")
                        .header("X-Ver-Os", "17")
                        .send()
                        .unwrap();

                    println!("Subscription");
                    println!("────────────────────────");
                    println!("Status: {}", response.status());

                    if let Some(value) = response.headers().get("content-type") {
                        println!("Content type: {:?}", value);
                    }

                    if let Some(value) = response.headers().get("content-length") {
                        println!("Content length: {:?}", value);
                    }

                    if let Some(value) = response.headers().get("profile-update-interval") {
                        println!("Update interval: {:?}", value);
                    }

                    if let Some(value) = response.headers().get("subscription-userinfo") {
                        println!("User info: {:?}", value);
                    }

                    if let Some(value) = response.headers().get("x-hwid-active") {
                        println!("HWID active: {:?}", value);
                    }

                    if let Some(value) = response.headers().get("x-hwid-limit") {
                        println!("HWID limit: {:?}", value);
                    }

                    if let Some(value) = response.headers().get("x-hwid-not-supported") {
                        println!("HWID supported: {:?}", value);
                    }
                }
                Some(SubscriptionCommand::List) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");
                    let subscription_file = xrayctl_dir.join("subscription.json");

                    let content = std::fs::read_to_string(&subscription_file).unwrap();

                    let subscription: serde_json::Value = serde_json::from_str(&content).unwrap();

                    let profiles = match subscription.as_array() {
                        Some(profiles) => profiles,
                        None => {
                            println!("Subscription is not an array");
                            return;
                        }
                    };

                    println!("Profiles");
                    println!("────────────────────────");

                    let mut profile_number = 0;
                    let mut current_group = String::from("Ungrouped");

                    for profile in profiles {
                        let remarks = profile
                            .get("remarks")
                            .and_then(|value| value.as_str())
                            .unwrap_or("Unnamed");

                        if remarks.starts_with("⬣↓") {
                            current_group = remarks.trim_start_matches("⬣↓").trim().to_string();

                            println!();
                            println!("{}", current_group);
                            println!("────────────────────────");

                            continue;
                        }

                        profile_number += 1;

                        let protocol = profile
                            .get("outbounds")
                            .and_then(|value| value.as_array())
                            .and_then(|outbounds| {
                                outbounds.iter().find(|outbound| {
                                    outbound.get("tag").and_then(|value| value.as_str())
                                        == Some("proxy")
                                })
                            })
                            .and_then(|proxy| proxy.get("protocol"))
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown");

                        println!("{}. {}", profile_number, remarks);

                        println!("   Protocol: {}", protocol);
                        println!("   Group: {}", current_group);
                    }

                    println!();
                    println!("Total profiles: {}", profile_number);
                }
                Some(SubscriptionCommand::Debug) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");
                    let subscription_file = xrayctl_dir.join("subscription.json");

                    let content = std::fs::read_to_string(&subscription_file).unwrap();

                    let subscription: serde_json::Value = serde_json::from_str(&content).unwrap();

                    let servers = match subscription.as_array() {
                        Some(servers) => servers,
                        None => {
                            println!("Subscription is not an array");
                            return;
                        }
                    };

                    for (index, profile) in servers.iter().enumerate() {
                        let remarks = profile
                            .get("remarks")
                            .and_then(|value| value.as_str())
                            .unwrap_or("Unnamed");

                        if index == 1 || index == 2 {
                            println!();
                            println!("===== PROFILE {} =====", index + 1);
                            println!("Remarks: {}", remarks);

                            println!("{}", serde_json::to_string_pretty(profile).unwrap());
                        }
                    }
                }
                Some(SubscriptionCommand::ShowProfile { index }) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");
                    let subscription_file = xrayctl_dir.join("subscription.json");

                    let content = match std::fs::read_to_string(&subscription_file) {
                        Ok(content) => content,
                        Err(error) => {
                            println!("Failed to read subscription: {error}");
                            return;
                        }
                    };

                    let subscription: serde_json::Value = match serde_json::from_str(&content) {
                        Ok(json) => json,
                        Err(error) => {
                            println!("Failed to parse subscription: {error}");
                            return;
                        }
                    };

                    let profiles = match subscription.as_array() {
                        Some(profiles) => profiles,
                        None => {
                            println!("Subscription is not an array");
                            return;
                        }
                    };

                    let mut profile_number = 0;

                    for profile in profiles {
                        let remarks = profile
                            .get("remarks")
                            .and_then(|value| value.as_str())
                            .unwrap_or("Unnamed");

                        if remarks.starts_with("⬣↓") {
                            continue;
                        }

                        profile_number += 1;

                        if profile_number != index {
                            continue;
                        }

                        let protocol = profile
                            .get("outbounds")
                            .and_then(|value| value.as_array())
                            .and_then(|outbounds| {
                                outbounds.iter().find(|outbound| {
                                    outbound.get("tag").and_then(|value| value.as_str())
                                        == Some("proxy")
                                })
                            })
                            .and_then(|proxy| proxy.get("protocol"))
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown");

                        println!("Profile #{}", index);
                        println!("────────────────────────");
                        println!("Name: {}", remarks);
                        println!("Protocol: {}", protocol);
                        println!();
                        println!("Raw configuration:");
                        println!("{}", serde_json::to_string_pretty(profile).unwrap());

                        return;
                    }

                    println!("Profile #{} not found", index);
                }
                Some(SubscriptionCommand::Use { index }) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");

                    let subscription_file = xrayctl_dir.join("subscription.json");
                    let active_file = xrayctl_dir.join("active.json");

                    let content = match std::fs::read_to_string(&subscription_file) {
                        Ok(content) => content,
                        Err(error) => {
                            println!("Failed to read subscription: {error}");
                            return;
                        }
                    };

                    let subscription: serde_json::Value = match serde_json::from_str(&content) {
                        Ok(json) => json,
                        Err(error) => {
                            println!("Failed to parse subscription: {error}");
                            return;
                        }
                    };

                    let profiles = match subscription.as_array() {
                        Some(profiles) => profiles,
                        None => {
                            println!("Subscription is not an array");
                            return;
                        }
                    };

                    let mut profile_number = 0;

                    for profile in profiles {
                        let remarks = profile
                            .get("remarks")
                            .and_then(|value| value.as_str())
                            .unwrap_or("Unnamed");

                        if remarks.starts_with("⬣↓") {
                            continue;
                        }

                        profile_number += 1;

                        if profile_number != index {
                            continue;
                        }

                        let formatted = serde_json::to_string_pretty(profile).unwrap();

                        if let Err(error) = std::fs::write(&active_file, formatted) {
                            println!("Failed to save active profile: {error}");
                            return;
                        }

                        let protocol = profile
                            .get("outbounds")
                            .and_then(|value| value.as_array())
                            .and_then(|outbounds| {
                                outbounds.iter().find(|outbound| {
                                    outbound.get("tag").and_then(|value| value.as_str())
                                        == Some("proxy")
                                })
                            })
                            .and_then(|proxy| proxy.get("protocol"))
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown");

                        println!("Profile selected:");
                        println!("────────────────────────");
                        println!("Number: {}", index);
                        println!("Name: {}", remarks);
                        println!("Protocol: {}", protocol);
                        println!();
                        println!("Saved to:");
                        println!("{}", active_file.display());

                        return;
                    }

                    println!("Profile #{} not found", index);
                }
                Some(SubscriptionCommand::Generate) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");

                    let active_file = xrayctl_dir.join("active.json");
                    let output_file = xrayctl_dir.join("xray.json");

                    let content = match std::fs::read_to_string(&active_file) {
                        Ok(content) => content,
                        Err(error) => {
                            println!("Failed to read active profile: {error}");
                            return;
                        }
                    };

                    let mut config: serde_json::Value = match serde_json::from_str(&content) {
                        Ok(json) => json,
                        Err(error) => {
                            println!("Failed to parse active profile: {error}");
                            return;
                        }
                    };

                    let proxy_address = match get_proxy_address(&config) {
                        Ok(address) => address,
                        Err(error) => {
                            println!("Failed to get proxy address: {error}");
                            return;
                        }
                    };

                    println!("Proxy address: {}", proxy_address);

                    let remarks = config
                        .get("remarks")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Unnamed");

                    let protocol = config
                        .get("outbounds")
                        .and_then(|value| value.as_array())
                        .and_then(|outbounds| {
                            outbounds.iter().find(|outbound| {
                                outbound.get("tag").and_then(|value| value.as_str())
                                    == Some("proxy")
                            })
                        })
                        .and_then(|proxy| proxy.get("protocol"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("unknown");

                    println!("Generating Xray configuration...");
                    println!("────────────────────────");
                    println!("Profile: {}", remarks);
                    println!("Protocol: {}", protocol);

                    config = match prepare_xray_config(config) {
                        Ok(config) => config,
                        Err(error) => {
                            println!("Failed to prepare Xray configuration: {error}");
                            return;
                        }
                    };

                    if let Some(inbounds) = config
                        .get_mut("inbounds")
                        .and_then(|value| value.as_array_mut())
                    {
                        let tun_exists = inbounds.iter().any(|inbound| {
                            inbound.get("protocol").and_then(|value| value.as_str()) == Some("tun")
                        });

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
                    }

                    let formatted = serde_json::to_string_pretty(&config).unwrap();

                    if let Err(error) = std::fs::write(&output_file, formatted) {
                        println!("Failed to save Xray configuration: {error}");
                        return;
                    }

                    println!();
                    println!("SOCKS: 127.0.0.1:9999");
                    println!("Saved to:");
                    println!("{}", output_file.display());
                }
                Some(SubscriptionCommand::Start { index }) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");

                    let active_file = xrayctl_dir.join("active.json");
                    let xray_file = xrayctl_dir.join("xray.json");

                    if let Some(index) = index {
                        let subscription_file = xrayctl_dir.join("subscription.json");

                        let content = match std::fs::read_to_string(&subscription_file) {
                            Ok(content) => content,
                            Err(error) => {
                                println!("Failed to read subscription: {error}");
                                return;
                            }
                        };

                        let subscription: serde_json::Value = match serde_json::from_str(&content) {
                            Ok(json) => json,
                            Err(error) => {
                                println!("Failed to parse subscription: {error}");
                                return;
                            }
                        };

                        let profiles = match subscription.as_array() {
                            Some(profiles) => profiles,
                            None => {
                                println!("Subscription is not an array");
                                return;
                            }
                        };

                        let mut profile_number = 0;
                        let mut selected_profile = None;

                        for profile in profiles {
                            let remarks = profile
                                .get("remarks")
                                .and_then(|value| value.as_str())
                                .unwrap_or("Unnamed");

                            if remarks.starts_with("⬣↓") {
                                continue;
                            }

                            profile_number += 1;

                            if profile_number == index {
                                selected_profile = Some(profile.clone());
                                break;
                            }
                        }

                        let profile = match selected_profile {
                            Some(profile) => profile,
                            None => {
                                println!("Profile #{} not found", index);
                                return;
                            }
                        };

                        let formatted = serde_json::to_string_pretty(&profile).unwrap();

                        if let Err(error) = std::fs::write(&active_file, formatted) {
                            println!("Failed to save active profile: {error}");
                            return;
                        }

                        println!("Profile selected: #{}", index);
                    }

                    let content = match std::fs::read_to_string(&active_file) {
                        Ok(content) => content,
                        Err(error) => {
                            println!("Failed to read active profile: {error}");
                            return;
                        }
                    };

                    let mut config: serde_json::Value = match serde_json::from_str(&content) {
                        Ok(json) => json,
                        Err(error) => {
                            println!("Failed to parse active profile: {error}");
                            return;
                        }
                    };

                    let remarks = config
                        .get("remarks")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Unnamed")
                        .to_string();

                    let protocol = config
                        .get("outbounds")
                        .and_then(|value| value.as_array())
                        .and_then(|outbounds| {
                            outbounds.iter().find(|outbound| {
                                outbound.get("tag").and_then(|value| value.as_str())
                                    == Some("proxy")
                            })
                        })
                        .and_then(|proxy| proxy.get("protocol"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("unknown");

                    println!();
                    println!("Preparing Xray...");
                    ui::separator();

                    ui::field("Profile", &remarks);
                    ui::field("Protocol", protocol);

                    if let Some(inbounds) = config
                        .get_mut("inbounds")
                        .and_then(|value| value.as_array_mut())
                    {
                        let mut socks_found = false;

                        for inbound in inbounds.iter_mut() {
                            let is_socks = inbound.get("protocol").and_then(|value| value.as_str())
                                == Some("socks");

                            if is_socks {
                                inbound["listen"] =
                                    serde_json::Value::String("127.0.0.1".to_string());

                                inbound["port"] = serde_json::Value::Number(9999.into());

                                socks_found = true;
                            }
                        }

                        if !socks_found {
                            println!("Error: SOCKS inbound not found.");
                            return;
                        }
                    } else {
                        println!("Error: inbounds not found.");
                        return;
                    }

                    config = match prepare_xray_config(config) {
                        Ok(config) => config,
                        Err(error) => {
                            println!("Failed to prepare Xray configuration: {error}");
                            return;
                        }
                    };

                    let formatted = serde_json::to_string_pretty(&config).unwrap();

                    if let Err(error) = std::fs::write(&xray_file, formatted) {
                        println!("Failed to save Xray configuration: {error}");
                        return;
                    }

                    ui::success("Configuration generated");
                    ui::field("SOCKS", "127.0.0.1:9999");

                    println!();
                    println!("Testing configuration...");

                    let test = Command::new("xray")
                        .arg("run")
                        .arg("-test")
                        .arg("-config")
                        .arg(&xray_file)
                        .output();

                    match test {
                        Ok(output) if output.status.success() => {
                            ui::success("Configuration valid");
                        }

                        Ok(output) => {
                            println!("Xray configuration is invalid.");

                            if !output.stderr.is_empty() {
                                println!("{}", String::from_utf8_lossy(&output.stderr));
                            }

                            return;
                        }

                        Err(error) => {
                            println!("Failed to run Xray: {error}");
                            return;
                        }
                    }

                    println!();
                    println!("Starting Xray...");
                    ui::separator();

                    let log_file = xrayctl_dir.join("xray.log");

                    let log = match std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log_file)
                    {
                        Ok(file) => file,
                        Err(error) => {
                            println!("Failed to open log: {error}");
                            return;
                        }
                    };

                    let log_error = match log.try_clone() {
                        Ok(file) => file,
                        Err(error) => {
                            println!("Failed to clone log: {error}");
                            return;
                        }
                    };

                    let child = match Command::new("xray")
                        .arg("run")
                        .arg("-config")
                        .arg(&xray_file)
                        .stdout(Stdio::from(log))
                        .stderr(Stdio::from(log_error))
                        .spawn()
                    {
                        Ok(child) => child,

                        Err(error) => {
                            println!("Failed to start Xray: {error}");
                            return;
                        }
                    };

                    let pid = child.id();

                    let proxy_address = match get_proxy_address(&config) {
                        Ok(address) => address,
                        Err(error) => {
                            println!("Failed to get proxy address: {error}");
                            return;
                        }
                    };

                    let proxy_ip = match resolve_proxy_address(&proxy_address) {
                        Ok(ip) => ip,
                        Err(error) => {
                            println!("Failed to resolve proxy address: {error}");
                            return;
                        }
                    };

                    let mut tun_ready = false;

                    for _ in 0..50 {
                        let status = Command::new("ip")
                            .args(["link", "show", "xray0"])
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status();

                        if matches!(status, Ok(status) if status.success()) {
                            tun_ready = true;
                            break;
                        }

                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }

                    if !tun_ready {
                        println!("Failed to wait for xray0: interface did not appear.");

                        let _ = Command::new("kill").arg(pid.to_string()).status();

                        return;
                    }

                    if let Err(error) = configure_tun(&proxy_ip) {
                        println!("Failed to configure TUN: {error}");

                        let _ = Command::new("kill").arg(pid.to_string()).status();

                        return;
                    }

                    let pid_file = xrayctl_dir.join("xray.pid");

                    if let Err(error) = std::fs::write(&pid_file, pid.to_string()) {
                        println!("Warning: failed to save PID: {error}");
                    }

                    let pid = child.id();

                    println!("Xray started successfully!");
                    println!();
                    println!("PID: {}", pid);
                    println!("SOCKS: 127.0.0.1:9999");
                    println!("Profile: {}", remarks);
                    println!("Config: {}", xray_file.display());
                    println!("Log: {}", log_file.display());
                }
                Some(SubscriptionCommand::Stop) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");

                    let pid_file = xrayctl_dir.join("xray.pid");

                    if !pid_file.exists() {
                        println!("Xray is not running.");
                        return;
                    }

                    let pid = match std::fs::read_to_string(&pid_file) {
                        Ok(pid) => match pid.trim().parse::<u32>() {
                            Ok(pid) => pid,
                            Err(_) => {
                                println!("Invalid PID file.");
                                let _ = std::fs::remove_file(&pid_file);
                                return;
                            }
                        },
                        Err(error) => {
                            println!("Failed to read PID file: {error}");
                            return;
                        }
                    };

                    println!("Stopping Xray...");
                    println!("PID: {}", pid);

                    let status = Command::new("kill").arg(pid.to_string()).status();

                    match status {
                        Ok(status) if status.success() => {
                            println!("Xray stopped.");

                            let _ = std::fs::remove_file(&pid_file);
                        }

                        Ok(_) => {
                            println!("Failed to stop Xray.");
                        }

                        Err(error) => {
                            println!("Failed to execute kill: {error}");
                        }
                    }
                }
                Some(SubscriptionCommand::Status) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");

                    let pid_file = xrayctl_dir.join("xray.pid");

                    ui::title("Xray");

                    if !pid_file.exists() {
                        ui::info("Stopped");
                        println!();
                        ui::field("SOCKS", "127.0.0.1:9999");
                        return;
                    }

                    let pid = match std::fs::read_to_string(&pid_file) {
                        Ok(pid) => match pid.trim().parse::<u32>() {
                            Ok(pid) => pid,
                            Err(_) => {
                                ui::error("Invalid PID file");
                                return;
                            }
                        },
                        Err(_) => {
                            ui::error("Failed to read PID file");
                            return;
                        }
                    };

                    let running = Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .status()
                        .map(|status| status.success())
                        .unwrap_or(false);

                    println!();

                    if running {
                        ui::info("Running");
                        println!();

                        ui::field("PID", pid);
                        ui::field("SOCKS", "127.0.0.1:9999");
                    } else {
                        ui::warning("Stopped");
                        println!();
                        ui::field("Reason", "PID file is stale");

                        let _ = std::fs::remove_file(&pid_file);
                    }
                }
                Some(SubscriptionCommand::Update) => {
                    let config_dir = dirs::config_dir().unwrap();
                    let xrayctl_dir = config_dir.join("xrayctl");
                    let config_file = xrayctl_dir.join("config.toml");

                    let toml = std::fs::read_to_string(&config_file).unwrap();

                    let config: Config = toml::from_str(&toml).unwrap();

                    println!("Updating: {}", config.subscription_url);

                    let client = reqwest::blocking::Client::new();

                    let response = client
                        .get(&config.subscription_url)
                        .header("User-Agent", "Happ/3.13.0")
                        .header("X-HWID", &config.hwid)
                        .header("X-Device-Os", "Android")
                        .header("X-Device-Locale", "ru")
                        .header("X-Device-Model", "xrayctl")
                        .header("X-Ver-Os", "17")
                        .send()
                        .unwrap();

                    println!("Status: {}", response.status());

                    let body = response.text().unwrap();

                    println!("Downloaded: {} bytes", body.len());

                    let subscription: serde_json::Value = match serde_json::from_str(&body) {
                        Ok(json) => json,
                        Err(error) => {
                            println!("Invalid subscription JSON: {error}");
                            return;
                        }
                    };

                    let subscription_file = xrayctl_dir.join("subscription.json");

                    let formatted = serde_json::to_string_pretty(&subscription).unwrap();

                    std::fs::write(&subscription_file, formatted).unwrap();

                    println!("Subscription saved:");
                    println!("{}", subscription_file.display());
                }

                None => {
                    println!("No subscription command specified.");
                }
            }
        }
    }
}
