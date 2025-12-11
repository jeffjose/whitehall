use anyhow::{Context, Result};
use crate::toolchain::Toolchain;

/// Device info
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub status: String,
}

/// Get list of connected devices
pub fn get_devices(toolchain: &Toolchain) -> Result<Vec<DeviceInfo>> {
    let output = toolchain
        .adb_cmd()?
        .args(["devices"])
        .output()
        .context("Failed to run 'adb devices'")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<DeviceInfo> = stdout
        .lines()
        .skip(1) // Skip header "List of devices attached"
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(DeviceInfo {
                    id: parts[0].to_string(),
                    status: parts[1].to_string(),
                })
            } else {
                None
            }
        })
        .filter(|d| d.status == "device") // Only online devices
        .collect();

    Ok(devices)
}

/// Find device by partial ID match
pub fn find_device<'a>(devices: &'a [DeviceInfo], query: &str) -> Result<&'a DeviceInfo> {
    let query_lower = query.to_lowercase();

    // Try prefix match first
    let matches: Vec<_> = devices
        .iter()
        .filter(|d| d.id.to_lowercase().starts_with(&query_lower))
        .collect();

    if matches.len() == 1 {
        return Ok(matches[0]);
    }

    if matches.len() > 1 {
        anyhow::bail!(
            "Ambiguous device ID '{}'. Matches:\n{}",
            query,
            matches.iter().map(|d| format!("  {}", d.id)).collect::<Vec<_>>().join("\n")
        );
    }

    // Try substring match
    let matches: Vec<_> = devices
        .iter()
        .filter(|d| d.id.to_lowercase().contains(&query_lower))
        .collect();

    if matches.len() == 1 {
        return Ok(matches[0]);
    }

    if matches.len() > 1 {
        anyhow::bail!(
            "Ambiguous device ID '{}'. Matches:\n{}",
            query,
            matches.iter().map(|d| format!("  {}", d.id)).collect::<Vec<_>>().join("\n")
        );
    }

    anyhow::bail!("No device found matching '{}'", query);
}

/// Resolve device - either find by query or auto-select if only one
pub fn resolve_device(toolchain: &Toolchain, query: Option<&str>) -> Result<String> {
    let devices = get_devices(toolchain)?;

    if devices.is_empty() {
        anyhow::bail!(
            "No devices connected. Please:\n  \
            1. Connect a device via USB with USB debugging enabled, or\n  \
            2. Start an emulator with 'whitehall emulator start'"
        );
    }

    match query {
        Some(q) => {
            let device = find_device(&devices, q)?;
            Ok(device.id.clone())
        }
        None => {
            if devices.len() == 1 {
                Ok(devices[0].id.clone())
            } else {
                anyhow::bail!(
                    "Multiple devices connected. Specify one:\n{}",
                    devices.iter().map(|d| format!("  {}", d.id)).collect::<Vec<_>>().join("\n")
                );
            }
        }
    }
}
