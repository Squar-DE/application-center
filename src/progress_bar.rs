use gtk::ProgressBar;
use glib::clone;
use regex::Regex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use std::sync::{Arc, Mutex};

#[allow(deprecated)]
pub async fn install_with_progress(package_name: &str, progress_bar: ProgressBar) -> Result<(), String> {
    // Set initial state
    progress_bar.set_fraction(0.0);
    progress_bar.set_text(Some("Starting installation..."));
    
    // Use a shared flag to track when the process is done
    let installation_done = Arc::new(Mutex::new(false));
    let installation_done_clone = installation_done.clone();
    
    // Start a separate task to pulse the progress bar periodically
    // This ensures UI responsiveness even if we're not getting output
    glib::MainContext::default().spawn_local(clone!(@weak progress_bar => async move {
        let mut pulse_counter = 0;
        
        while !*installation_done_clone.lock().unwrap() {
            // Only pulse if we're still at 0%
            if progress_bar.fraction() < 0.01 {
                progress_bar.pulse();
                
                // Update text occasionally to show activity
                pulse_counter += 1;
                if pulse_counter % 5 == 0 {
                    let dots = ".".repeat((pulse_counter / 5) % 4);
                    progress_bar.set_text(Some(&format!("Installing{}", dots)));
                }
            }
            
            // Sleep to avoid hogging the UI thread
            sleep(Duration::from_millis(200)).await;
        }
    }));

    // Determine which command to run
    let args = if package_name.is_empty() {
        // If package_name is empty, perform system update
        vec!["pacman", "-Syu", "--noconfirm"]
    } else {
        // Otherwise install the specific package
        vec!["pacman", "-S", "--noconfirm", package_name]
    };

    // Spawn the pacman process
    let mut child = Command::new("pkexec")
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn pacman: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Common patterns seen in pacman output
    let re_standard = Regex::new(r"\[#*-*\]\s+(\d+)%").unwrap();
    let re_download = Regex::new(r"downloading.*?\((\d+)%\)").unwrap();
    
    // Process output in a separate task
    let _progress_handle = glib::MainContext::default().spawn_local(clone!(@weak progress_bar => async move {
        let mut has_progress = false;
        
        loop {
            tokio::select! {
                result = stdout_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            // Try matching progress patterns
                            let mut found_progress = false;
                            
                            if let Some(caps) = re_standard.captures(&line) {
                                if let Some(percent_match) = caps.get(1) {
                                    if let Ok(percent) = percent_match.as_str().parse::<f64>() {
                                        let fraction = percent / 100.0;
                                        progress_bar.set_fraction(fraction);
                                        progress_bar.set_text(Some(&format!("{}%", percent)));
                                        has_progress = true;
                                        found_progress = true;
                                    }
                                }
                            } else if let Some(caps) = re_download.captures(&line) {
                                if let Some(percent_match) = caps.get(1) {
                                    if let Ok(percent) = percent_match.as_str().parse::<f64>() {
                                        let fraction = percent / 100.0;
                                        progress_bar.set_fraction(fraction);
                                        progress_bar.set_text(Some(&format!("{}%", percent)));
                                        has_progress = true;
                                        found_progress = true;
                                    }
                                }
                            }
                            
                            // Print to console for debugging
                            eprintln!("STDOUT: {}", line);
                        },
                        Ok(None) => break,
                        Err(e) => eprintln!("Error reading stdout: {}", e),
                    }
                },
                result = stderr_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            // Check stderr for progress patterns
                            if let Some(caps) = re_standard.captures(&line) {
                                if let Some(percent_match) = caps.get(1) {
                                    if let Ok(percent) = percent_match.as_str().parse::<f64>() {
                                        let fraction = percent / 100.0;
                                        progress_bar.set_fraction(fraction);
                                        progress_bar.set_text(Some(&format!("{}%", percent)));
                                        has_progress = true;
                                    }
                                }
                            } else if let Some(caps) = re_download.captures(&line) {
                                if let Some(percent_match) = caps.get(1) {
                                    if let Ok(percent) = percent_match.as_str().parse::<f64>() {
                                        let fraction = percent / 100.0;
                                        progress_bar.set_fraction(fraction);
                                        progress_bar.set_text(Some(&format!("{}%", percent)));
                                        has_progress = true;
                                    }
                                }
                            }
                            
                            // Print to console for debugging
                            eprintln!("STDERR: {}", line);
                        },
                        Ok(None) => break,
                        Err(e) => eprintln!("Error reading stderr: {}", e),
                    }
                },
                else => break,
            }
        }
        
        // Always show completion on exit
        if !has_progress || progress_bar.fraction() < 0.99 {
            progress_bar.set_fraction(1.0);
            progress_bar.set_text(Some("100%"));
        }
    }));

    // Wait for the installation to complete
    let status = child.wait().await.map_err(|e| format!("Installation failed: {}", e))?;
    
    // Signal the progress bar animation task to stop
    *installation_done.lock().unwrap() = true;
    
    // Ensure progress is at 100% 
    progress_bar.set_fraction(1.0);
    progress_bar.set_text(Some("100%"));
    
    // A small delay to ensure UI updates
    sleep(Duration::from_millis(100)).await;
    
    if status.success() {
        Ok(())
    } else {
        Err(format!("Installation exited with code: {:?}", status.code()))
    }
}

#[allow(deprecated)]
pub async fn uninstall_with_progress(package_name: &str, progress_bar: ProgressBar) -> Result<(), String> {
    // Set initial state
    progress_bar.set_fraction(0.0);
    progress_bar.set_text(Some("Starting uninstallation..."));
    
    // Use a shared flag to track when the process is done
    let uninstall_done = Arc::new(Mutex::new(false));
    let uninstall_done_clone = uninstall_done.clone();
    
    // Start a separate task to pulse the progress bar periodically
    glib::MainContext::default().spawn_local(clone!(@weak progress_bar => async move {
        let mut pulse_counter = 0;
        
        while !*uninstall_done_clone.lock().unwrap() {
            // Only pulse if we're still at 0%
            if progress_bar.fraction() < 0.01 {
                progress_bar.pulse();
                
                // Update text occasionally to show activity
                pulse_counter += 1;
                if pulse_counter % 5 == 0 {
                    let dots = ".".repeat((pulse_counter / 5) % 4);
                    progress_bar.set_text(Some(&format!("Uninstalling{}", dots)));
                }
            }
            
            // Sleep to avoid hogging the UI thread
            sleep(Duration::from_millis(200)).await;
        }
    }));

    // Spawn the pacman uninstall process
    let mut child = Command::new("pkexec")
        .args(&["pacman", "-R", "--noconfirm", package_name])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn pacman: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Process output in a separate task
    let progress_handle = glib::MainContext::default().spawn_local(clone!(@weak progress_bar => async move {
        loop {
            tokio::select! {
                result = stdout_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            eprintln!("STDOUT: {}", line);
                        },
                        Ok(None) => break,
                        Err(e) => eprintln!("Error reading stdout: {}", e),
                    }
                },
                result = stderr_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            eprintln!("STDERR: {}", line);
                        },
                        Ok(None) => break,
                        Err(e) => eprintln!("Error reading stderr: {}", e),
                    }
                },
                else => break,
            }
        }
        
        // Show completion
        progress_bar.set_fraction(1.0);
        progress_bar.set_text(Some("100%"));
    }));

    // Wait for the uninstallation to complete
    let status = child.wait().await.map_err(|e| format!("Uninstallation failed: {}", e))?;
    
    // Signal the progress bar animation task to stop
    *uninstall_done.lock().unwrap() = true;
    
    // Ensure progress is at 100% 
    progress_bar.set_fraction(1.0);
    progress_bar.set_text(Some("100%"));
    
    // A small delay to ensure UI updates
    sleep(Duration::from_millis(100)).await;
    
    if status.success() {
        Ok(())
    } else {
        Err(format!("Uninstallation exited with code: {:?}", status.code()))
    }
}

// Function to check if a package is installed
pub async fn is_package_installed(package_name: &str) -> Result<bool, String> {
    let output = Command::new("pacman")
        .args(["-Q", package_name])
        .output()
        .await
        .map_err(|e| format!("Failed to check package status: {}", e))?;
    
    Ok(output.status.success())
}
