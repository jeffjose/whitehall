use anyhow::{Context, Result};
use colored::Colorize;
use crate::toolchain::Toolchain;

/// Device info
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub short_id: String,
    pub status: String,
    pub model: Option<String>,
    pub product: Option<String>,
}

/// Generate a short ID from device ID (first 8 chars of hash)
fn generate_short_id(id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    format!("{:08x}", hasher.finish() as u32)
}

/// Get list of connected devices with detailed info
pub fn get_devices(toolchain: &Toolchain) -> Result<Vec<DeviceInfo>> {
    let output = toolchain
        .adb_cmd()?
        .args(["devices", "-l"])
        .output()
        .context("Failed to run 'adb devices -l'")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<DeviceInfo> = stdout
        .lines()
        .skip(1) // Skip header "List of devices attached"
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let id = parts[0].to_string();
                let status = parts[1].to_string();

                // Parse key:value pairs for model and product
                let mut model = None;
                let mut product = None;
                for part in &parts[2..] {
                    if let Some(val) = part.strip_prefix("model:") {
                        model = Some(val.to_string());
                    } else if let Some(val) = part.strip_prefix("product:") {
                        product = Some(val.to_string());
                    }
                }

                let short_id = generate_short_id(&id);

                Some(DeviceInfo {
                    id,
                    short_id,
                    status,
                    model,
                    product,
                })
            } else {
                None
            }
        })
        .filter(|d| d.status == "device") // Only online devices
        .collect();

    Ok(devices)
}

/// Find device by partial ID match (matches short_id prefix, id prefix, or id substring)
pub fn find_device<'a>(devices: &'a [DeviceInfo], query: &str) -> Result<&'a DeviceInfo> {
    let query_lower = query.to_lowercase();

    // Try short_id prefix match first
    let matches: Vec<_> = devices
        .iter()
        .filter(|d| d.short_id.starts_with(&query_lower))
        .collect();

    if matches.len() == 1 {
        return Ok(matches[0]);
    }

    if matches.len() > 1 {
        anyhow::bail!(
            "Ambiguous device ID '{}'. Matches:\n{}",
            query,
            matches.iter().map(|d| format!("  {} {}", d.short_id, d.id)).collect::<Vec<_>>().join("\n")
        );
    }

    // Try full id prefix match
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
            matches.iter().map(|d| format!("  {} {}", d.short_id, d.id)).collect::<Vec<_>>().join("\n")
        );
    }

    // Try full id substring match
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
            matches.iter().map(|d| format!("  {} {}", d.short_id, d.id)).collect::<Vec<_>>().join("\n")
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
                    devices.iter().map(|d| format!("  {} {}", d.short_id, d.id)).collect::<Vec<_>>().join("\n")
                );
            }
        }
    }
}

/// List connected devices
pub fn execute_list() -> Result<()> {
    let toolchain = Toolchain::new()?;
    let devices = get_devices(&toolchain)?;

    if devices.is_empty() {
        println!("{}", "No devices connected.".yellow());
        println!();
        println!("Connect a device via USB or start an emulator:");
        println!("  whitehall emulator start <id>");
    } else {
        // Calculate column widths
        let max_id_len = devices.iter().map(|d| d.id.len()).max().unwrap_or(0);

        // Print header
        println!(
            "{:<8}  {:<width$}  {}",
            "ID".dimmed(),
            "DEVICE".dimmed(),
            "MODEL".dimmed(),
            width = max_id_len
        );

        for device in &devices {
            let model = device.model.as_deref().unwrap_or("-");
            println!(
                "{}  {:<width$}  {}",
                device.short_id.yellow(),
                device.id,
                model,
                width = max_id_len
            );
        }
    }

    Ok(())
}
