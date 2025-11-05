use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use super::Platform;

const MAX_RETRIES: usize = 3;

/// Download a file from URL with progress bar
///
/// Returns the path to the downloaded file
pub fn download_with_progress(url: &str, dest_path: &Path) -> Result<PathBuf> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(600)) // 10 minute timeout
        .build()
        .context("Failed to create HTTP client")?;

    println!("Downloading from: {}", url);

    let response = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to download from {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    // Get content length for progress bar
    let total_size = response.content_length().unwrap_or(0);

    // Create progress bar
    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(format!("Downloading {}", dest_path.file_name().unwrap().to_str().unwrap()));
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_message(format!("Downloading {} (unknown size)", dest_path.file_name().unwrap().to_str().unwrap()));
        pb
    };

    // Create parent directory
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Download to temporary file first
    let temp_path = dest_path.with_extension("tmp");
    let mut dest_file = File::create(&temp_path)
        .with_context(|| format!("Failed to create file: {}", temp_path.display()))?;

    // Stream download with progress
    let mut downloaded: u64 = 0;
    let mut buffer = [0; 8192];
    let mut reader = response;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        dest_file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download complete");

    // Move temp file to final location
    fs::rename(&temp_path, dest_path)
        .with_context(|| format!("Failed to rename {} to {}", temp_path.display(), dest_path.display()))?;

    Ok(dest_path.to_path_buf())
}

/// Download with automatic retry on failure
///
/// Prompts user to retry if download fails
/// Optionally verifies checksum if provided
pub fn download_with_retry(url: &str, dest_path: &Path) -> Result<PathBuf> {
    download_with_retry_and_checksum(url, dest_path, None)
}

/// Download with retry and optional checksum verification
///
/// # Arguments
/// * `url` - URL to download from
/// * `dest_path` - Destination file path
/// * `expected_checksum` - Optional SHA256 checksum to verify
pub fn download_with_retry_and_checksum(
    url: &str,
    dest_path: &Path,
    expected_checksum: Option<&str>
) -> Result<PathBuf> {
    let mut attempt = 1;

    loop {
        match download_with_progress(url, dest_path) {
            Ok(path) => {
                // Verify checksum if provided
                if let Some(expected) = expected_checksum {
                    if let Err(e) = verify_checksum(&path, expected) {
                        eprintln!("\n{} {}", "error:".red().bold(), e);

                        // Clean up invalid file
                        let _ = fs::remove_file(&path);

                        if attempt >= MAX_RETRIES {
                            eprintln!("\n{} Maximum retry attempts reached.", "error:".red().bold());
                            return Err(e);
                        }

                        print!("\n{} Retry download? [Y/n]: ", "?".yellow().bold());
                        io::stdout().flush()?;

                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        let input = input.trim().to_lowercase();

                        if input == "n" || input == "no" {
                            eprintln!("{} Download cancelled by user.", "info:".cyan());
                            return Err(e);
                        }

                        println!("\n{} Retrying download...", "info:".cyan().bold());
                        attempt += 1;
                        continue;
                    }
                }

                return Ok(path);
            },
            Err(e) => {
                eprintln!("\n{} Download failed (attempt {}/{}): {}",
                    "error:".red().bold(), attempt, MAX_RETRIES, e);

                if attempt >= MAX_RETRIES {
                    eprintln!("\n{} Maximum retry attempts reached.", "error:".red().bold());
                    return Err(e);
                }

                // Ask user if they want to retry
                print!("\n{} Retry download? [Y/n]: ", "?".yellow().bold());
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();

                if input == "n" || input == "no" {
                    eprintln!("{} Download cancelled by user.", "info:".cyan());
                    return Err(e);
                }

                println!("\n{} Retrying download...", "info:".cyan().bold());
                attempt += 1;

                // Clean up partial download if it exists
                if dest_path.exists() {
                    let _ = fs::remove_file(dest_path);
                }
                let temp_path = dest_path.with_extension("tmp");
                if temp_path.exists() {
                    let _ = fs::remove_file(&temp_path);
                }
            }
        }
    }
}

/// Extract a .tar.gz archive
///
/// Used for Java and Gradle downloads
pub fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    println!("Extracting {}...", archive_path.file_name().unwrap().to_str().unwrap());

    let tar_gz = File::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;

    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    // Create destination directory
    fs::create_dir_all(dest_dir)?;

    // Extract with progress
    let entries = archive.entries()?;
    let pb = ProgressBar::new_spinner();
    pb.set_message("Extracting files...");

    for (i, entry_result) in entries.enumerate() {
        let mut entry = entry_result?;
        entry.unpack_in(dest_dir)?;

        if i % 10 == 0 {
            pb.set_message(format!("Extracting files... ({})", i));
        }
    }

    pb.finish_with_message("Extraction complete");

    Ok(())
}

/// Extract a .zip archive
///
/// Used for Android SDK cmdline-tools
pub fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    println!("Extracting {}...", archive_path.file_name().unwrap().to_str().unwrap());

    let file = File::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;

    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to read ZIP archive: {}", archive_path.display()))?;

    // Create destination directory
    fs::create_dir_all(dest_dir)?;

    // Extract with progress
    let pb = ProgressBar::new(archive.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("Extracting files...");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.name());

        if file.name().ends_with('/') {
            // Directory
            fs::create_dir_all(&outpath)?;
        } else {
            // File
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Extraction complete");

    Ok(())
}

/// Verify file checksum (SHA256)
pub fn verify_checksum(file_path: &Path, expected: &str) -> Result<()> {
    use sha2::{Digest, Sha256};

    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let result = hasher.finalize();
    let actual = format!("{:x}", result);

    if actual != expected {
        anyhow::bail!(
            "Checksum mismatch for {}:\n  Expected: {}\n  Actual:   {}",
            file_path.display(),
            expected,
            actual
        );
    }

    println!("✓ Checksum verified");
    Ok(())
}

/// Get Java download URL for a specific version and platform
pub fn get_java_download_url(version: &str, platform: Platform) -> Result<String> {
    let (os, arch) = platform.as_download_strings();

    // Adoptium/Temurin API
    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse",
        version, os, arch
    );

    Ok(url)
}

/// Get Gradle download URL for a specific version
pub fn get_gradle_download_url(version: &str) -> String {
    format!(
        "https://services.gradle.org/distributions/gradle-{}-bin.zip",
        version
    )
}

/// Get Android cmdline-tools download URL for platform
pub fn get_android_cmdline_tools_url(platform: Platform) -> Result<String> {
    let os_name = if platform.is_linux() {
        "linux"
    } else if platform.is_macos() {
        "mac"
    } else {
        anyhow::bail!("Unsupported platform for Android SDK");
    };

    // Latest version number (update as needed)
    let version = "9477386";

    Ok(format!(
        "https://dl.google.com/android/repository/commandlinetools-{}-{}_latest.zip",
        os_name, version
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_url_generation() {
        let platform = Platform::LinuxX64;
        let url = get_java_download_url("21", platform).unwrap();
        assert!(url.contains("21"));
        assert!(url.contains("linux"));
        assert!(url.contains("x64"));
    }

    #[test]
    fn test_gradle_url_generation() {
        let url = get_gradle_download_url("8.4");
        assert_eq!(url, "https://services.gradle.org/distributions/gradle-8.4-bin.zip");
    }

    #[test]
    fn test_android_cmdline_tools_url() {
        let url = get_android_cmdline_tools_url(Platform::LinuxX64).unwrap();
        assert!(url.contains("linux"));
        assert!(url.contains("commandlinetools"));
    }
}
