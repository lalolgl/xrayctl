use crate::config::Config;
use serde_json::Value;

pub fn fetch(config: &Config) -> Result<Value, String> {
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
        .map_err(|error| format!("Failed to fetch subscription: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Subscription request failed with status: {}",
            response.status()
        ));
    }

    let body = response
        .text()
        .map_err(|error| format!("Failed to read response body: {error}"))?;

    serde_json::from_str(&body).map_err(|error| format!("Invalid subscription JSON: {error}"))
}

pub fn info(config: &Config) -> Result<reqwest::blocking::Response, String> {
    let client = reqwest::blocking::Client::new();

    client
        .get(&config.subscription_url)
        .header("User-Agent", "Happ/3.13.0")
        .header("X-Device-Os", "Android")
        .header("X-Device-Locale", "ru")
        .header("X-Device-Model", "xrayctl")
        .header("X-Ver-Os", "17")
        .send()
        .map_err(|error| format!("Failed to fetch subscription info: {error}"))
}

pub fn save(subscription: &Value) -> Result<std::path::PathBuf, String> {
    let subscription_file = crate::config::subscription_file()?;

    let formatted = serde_json::to_string_pretty(subscription)
        .map_err(|error| format!("Failed to format subscription: {error}"))?;

    std::fs::write(&subscription_file, formatted)
        .map_err(|error| format!("Failed to save subscription: {error}"))?;

    Ok(subscription_file)
}
