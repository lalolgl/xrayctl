use serde_json::Value;

pub fn find_proxy_outbound(profile: &Value) -> Option<&Value> {
    profile
        .get("outbounds")
        .and_then(|value| value.as_array())
        .and_then(|outbounds| {
            outbounds.iter().find(|outbound| {
                outbound.get("tag").and_then(|value| value.as_str()) == Some("proxy")
            })
        })
}

pub fn get_proxy_address(profile: &Value) -> Result<String, String> {
    let proxy =
        find_proxy_outbound(profile).ok_or_else(|| "Proxy outbound not found".to_string())?;

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

pub fn get_proxy_port(profile: &Value) -> Result<u16, String> {
    let proxy =
        find_proxy_outbound(profile).ok_or_else(|| "Proxy outbound not found".to_string())?;

    let settings = proxy
        .get("settings")
        .ok_or_else(|| "Proxy settings not found".to_string())?;

    // Hysteria
    if let Some(port) = settings.get("port").and_then(|value| value.as_u64()) {
        return u16::try_from(port).map_err(|_| "Invalid proxy port".to_string());
    }

    // VLESS
    if let Some(port) = settings
        .get("vnext")
        .and_then(|value| value.as_array())
        .and_then(|vnext| vnext.first())
        .and_then(|server| server.get("port"))
        .and_then(|value| value.as_u64())
    {
        return u16::try_from(port).map_err(|_| "Invalid proxy port".to_string());
    }

    Err("Proxy port not found".to_string())
}

pub fn get_protocol(profile: &Value) -> &str {
    find_proxy_outbound(profile)
        .and_then(|proxy| proxy.get("protocol"))
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
}
