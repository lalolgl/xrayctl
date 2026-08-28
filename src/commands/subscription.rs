use crate::cli::SubscriptionCommand;
use crate::config;
use crate::config::Config;
use crate::subscription;
use crate::subscription_client;
use crate::ui;
use crate::xray;
use std::process::Command;

pub fn run(command: Option<SubscriptionCommand>) {
    match command {
        Some(SubscriptionCommand::Add { url }) => {
            let config = Config {
                subscription_url: url,
                hwid: String::from("xrayctl-test-hwid"),
            };

            let xrayctl_dir = match config::config_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    println!("Failed to get config directory: {error}");
                    return;
                }
            };

            if let Err(error) = std::fs::create_dir_all(&xrayctl_dir) {
                println!("Failed to create config directory: {error}");
                return;
            }

            let config_file = xrayctl_dir.join("config.toml");

            let toml = match toml::to_string(&config) {
                Ok(toml) => toml,
                Err(error) => {
                    println!("Failed to serialize config: {error}");
                    return;
                }
            };

            if let Err(error) = std::fs::write(&config_file, toml) {
                println!("Failed to save config: {error}");
                return;
            }

            println!("Subscription added:");
            println!("{}", config.subscription_url);
        }
        Some(SubscriptionCommand::Fetch) => {
            let config = match config::load() {
                Ok(config) => config,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

            let subscription = match subscription_client::fetch(&config) {
                Ok(subscription) => subscription,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

            println!("Valid subscription JSON!");

            let subscription_file = match subscription_client::save(&subscription) {
                Ok(path) => path,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

            println!("Subscription saved:");
            println!("{}", subscription_file.display());
        }
        Some(SubscriptionCommand::Show) => {
            let config = match config::load() {
                Ok(config) => config,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };
            println!("{}", config.subscription_url);
        }
        Some(SubscriptionCommand::Info) => {
            let config = match config::load() {
                Ok(config) => config,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

            let response = match subscription_client::info(&config) {
                Ok(response) => response,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

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
            let subscription_file = match config::subscription_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get subscription file: {error}");
                    return;
                }
            };

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

                let protocol = subscription::get_protocol(profile);

                println!("{}. {}", profile_number, remarks);
                println!("   Protocol: {}", protocol);
                println!("   Group: {}", current_group);
            }

            println!();
            println!("Total profiles: {}", profile_number);
        }
        Some(SubscriptionCommand::Debug) => {
            let subscription_file = match config::subscription_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get subscription file: {error}");
                    return;
                }
            };

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
            let subscription_file = match config::subscription_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get subscription file: {error}");
                    return;
                }
            };

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

                let protocol = subscription::get_protocol(profile);

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
            let subscription_file = match config::subscription_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get subscription file: {error}");
                    return;
                }
            };

            let active_file = match config::active_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get active file: {error}");
                    return;
                }
            };

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

                let formatted = match serde_json::to_string_pretty(profile) {
                    Ok(formatted) => formatted,
                    Err(error) => {
                        println!("Failed to format profile: {error}");
                        return;
                    }
                };

                if let Err(error) = std::fs::write(&active_file, formatted) {
                    println!("Failed to save active profile: {error}");
                    return;
                }

                let protocol = subscription::get_protocol(profile);

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
            let active_file = match config::active_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get active file: {error}");
                    return;
                }
            };

            let output_file = match config::xray_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get Xray config file: {error}");
                    return;
                }
            };

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

            let proxy_address = match subscription::get_proxy_address(&config) {
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

            let protocol = subscription::get_protocol(&config);

            println!("Generating Xray configuration...");
            println!("────────────────────────");
            println!("Profile: {}", remarks);
            println!("Protocol: {}", protocol);

            config = match xray::prepare_xray_config(config) {
                Ok(config) => config,
                Err(error) => {
                    println!("Failed to prepare Xray configuration: {error}");
                    return;
                }
            };

            let formatted = match serde_json::to_string_pretty(&config) {
                Ok(formatted) => formatted,
                Err(error) => {
                    println!("Failed to format Xray configuration: {error}");
                    return;
                }
            };

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
            let config_dir = match config::config_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    println!("Failed to get config directory: {error}");
                    return;
                }
            };

            let active_file = config_dir.join("active.json");
            let xray_file = config_dir.join("xray.json");

            if let Some(index) = index {
                let subscription_file = config_dir.join("subscription.json");

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

                let formatted = match serde_json::to_string_pretty(&profile) {
                    Ok(formatted) => formatted,
                    Err(error) => {
                        println!("Failed to format profile: {error}");
                        return;
                    }
                };

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

            let protocol = subscription::get_protocol(&config);

            println!();
            println!("Preparing Xray...");
            ui::separator();

            ui::field("Profile", &remarks);
            ui::field("Protocol", protocol);

            config = match xray::prepare_xray_config(config) {
                Ok(config) => config,
                Err(error) => {
                    println!("Failed to prepare Xray configuration: {error}");
                    return;
                }
            };

            let formatted = match serde_json::to_string_pretty(&config) {
                Ok(formatted) => formatted,
                Err(error) => {
                    println!("Failed to format Xray configuration: {error}");
                    return;
                }
            };

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

            let pid_file = match config::pid_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get PID file: {error}");
                    return;
                }
            };

            if let Some(pid) = std::fs::read_to_string(&pid_file)
                .ok()
                .and_then(|value| value.trim().parse::<u32>().ok())
            {
                if xray::is_running(pid) {
                    println!("Xray is already running (PID {}).", pid);
                    return;
                }

                let _ = std::fs::remove_file(&pid_file);
            }

            let proxy_address = match subscription::get_proxy_address(&config) {
                Ok(address) => address,
                Err(error) => {
                    println!("Failed to get proxy address: {error}");
                    return;
                }
            };

            let proxy_ip = match xray::resolve_proxy_address(&proxy_address) {
                Ok(ip) => ip,
                Err(error) => {
                    println!("Failed to resolve proxy address: {error}");
                    return;
                }
            };

            println!();
            println!("Starting Xray...");
            ui::separator();

            let log_file = match config::log_file() {
                Ok(path) => path,
                Err(error) => {
                    println!("Failed to get log file: {error}");
                    return;
                }
            };

            let child = match xray::start(&xray_file, &log_file) {
                Ok(child) => child,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

            let pid = child.id();

            if let Err(error) = xray::wait_for_tun("xray0") {
                println!("Failed to wait for xray0: {error}");

                let _ = xray::stop(pid);
                return;
            }

            if let Err(error) = xray::configure_tun(&proxy_ip) {
                println!("Failed to configure TUN: {error}");

                let _ = xray::stop(pid);
                return;
            }

            if let Err(error) = std::fs::write(&pid_file, pid.to_string()) {
                println!("Warning: failed to save PID: {error}");
            }

            ui::success("Xray started");

            println!();
            ui::field("PID", pid);
            ui::field("SOCKS", "127.0.0.1:9999");
            ui::field("Profile", &remarks);
            ui::field("Config", xray_file.display());
            ui::field("Log", log_file.display());
        }
        Some(SubscriptionCommand::Stop) => {
            let config_dir = match config::config_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    println!("Failed to get config directory: {error}");
                    return;
                }
            };

            let pid_file = config_dir.join("xray.pid");

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
            let config_dir = match config::config_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    println!("Failed to get config directory: {error}");
                    return;
                }
            };

            let pid_file = config_dir.join("xray.pid");

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
            let config = match config::load() {
                Ok(config) => config,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

            println!("Updating: {}", config.subscription_url);

            let subscription = match subscription_client::fetch(&config) {
                Ok(subscription) => subscription,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

            let subscription_file = match subscription_client::save(&subscription) {
                Ok(path) => path,
                Err(error) => {
                    println!("{error}");
                    return;
                }
            };

            println!("Subscription saved:");
            println!("{}", subscription_file.display());
        }
        None => {
            println!("No subscription command specified.");
        }
    }
}
