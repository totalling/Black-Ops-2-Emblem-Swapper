use std::process::Command;
use std::time::Duration;

use serde::Serialize;

use crate::paths::PROXY_PORT;

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub status: String,
    pub detail: String,
}

fn check(name: &str, status: &str, detail: impl Into<String>) -> DiagnosticCheck {
    DiagnosticCheck {
        name: name.to_string(),
        status: status.to_string(),
        detail: detail.into(),
    }
}

fn run_powershell(script: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

async fn check_proxy_listening() -> DiagnosticCheck {
    let addr = format!("127.0.0.1:{PROXY_PORT}");
    match tokio::time::timeout(Duration::from_millis(500), tokio::net::TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => check(
            "Proxy running",
            "ok",
            format!("Something is listening on port {PROXY_PORT}."),
        ),
        _ => check(
            "Proxy running",
            "fail",
            format!(
                "Nothing is listening on port {PROXY_PORT} right now. The app's proxy isn't \
                 running, or it failed to start (another program may already be using that port). \
                 Try restarting the app."
            ),
        ),
    }
}

fn check_lan_ip() -> DiagnosticCheck {
    let Some(ip) = crate::netinfo::get_lan_ip() else {
        return check(
            "LAN IP detected",
            "fail",
            "Couldn't detect a LAN IP address at all. Check that your PC is actually connected to a network.",
        );
    };

    let count = run_powershell(
        "$r = Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue; \
         if ($r) { $m = ($r | Measure-Object -Property RouteMetric -Minimum).Minimum; \
         @($r | Where-Object { $_.RouteMetric -eq $m }).Count } else { 0 }",
    )
    .and_then(|s| s.trim().parse::<u32>().ok());

    match count {
        Some(c) if c > 1 => check(
            "LAN IP detected",
            "warn",
            format!(
                "Currently showing {ip}, but {c} network interfaces are tied for priority \
                 (for example Ethernet and WiFi both active). Which IP this app shows can \
                 change between launches. Make sure the PS5's proxy setting matches whatever \
                 IP is shown here right now, not an old one."
            ),
        ),
        _ => check("LAN IP detected", "ok", format!("{ip}, no ambiguity detected.")),
    }
}

fn check_network_category() -> DiagnosticCheck {
    let Some(json) = run_powershell(
        "Get-NetConnectionProfile | Select-Object InterfaceAlias, NetworkCategory | ConvertTo-Json -Compress",
    ) else {
        return check(
            "Network type",
            "unknown",
            "Couldn't determine whether your network is set to Private or Public.",
        );
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) else {
        return check("Network type", "unknown", "Couldn't parse network category info.");
    };
    let items: Vec<serde_json::Value> = match value {
        serde_json::Value::Array(a) => a,
        other => vec![other],
    };

    let publics: Vec<String> = items
        .iter()
        .filter(|v| v["NetworkCategory"].as_str() == Some("Public"))
        .filter_map(|v| v["InterfaceAlias"].as_str().map(|s| s.to_string()))
        .collect();

    if publics.is_empty() {
        check("Network type", "ok", "All active networks are set to Private.")
    } else {
        check(
            "Network type",
            "warn",
            format!(
                "{} is set to Public. Windows Firewall blocks a lot more on Public networks, \
                 which can silently stop the PS5 from reaching this app. Switch it to Private \
                 in Settings > Network & Internet.",
                publics.join(", ")
            ),
        )
    }
}

fn check_firewall_rule() -> DiagnosticCheck {
    let script = "Get-NetFirewallRule -DisplayName 'Black Ops 2 Emblem Swapper' -ErrorAction SilentlyContinue \
                   | Select-Object Enabled, Action, Profile | ConvertTo-Json -Compress";
    let Some(json) = run_powershell(script) else {
        return check(
            "Firewall rule",
            "warn",
            "No firewall rule named \"Black Ops 2 Emblem Swapper\" was found. Windows Firewall \
             may be silently blocking the PS5's connection. See the Troubleshooting section \
             below for the fix.",
        );
    };

    let value: serde_json::Value = serde_json::from_str(&json).unwrap_or_default();
    let enabled = value["Enabled"].as_bool().unwrap_or(false) || value["Enabled"].as_str() == Some("True");
    let action = value["Action"].as_str().unwrap_or("");

    if enabled && action.eq_ignore_ascii_case("Allow") {
        check("Firewall rule", "ok", "An enabled Allow rule exists for this app.")
    } else {
        check(
            "Firewall rule",
            "warn",
            "A rule exists but isn't both Enabled and set to Allow. See the Troubleshooting \
             section below for the fix.",
        )
    }
}

pub async fn run() -> Vec<DiagnosticCheck> {
    vec![
        check_proxy_listening().await,
        check_lan_ip(),
        check_network_category(),
        check_firewall_rule(),
    ]
}
