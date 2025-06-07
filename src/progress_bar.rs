use gtk::ProgressBar;
use glib::{clone, idle_add_local, ControlFlow};
use regex::Regex;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;

pub async fn install_with_progress(package_name: &str, progress_bar: ProgressBar) -> Result<(), String> {
    progress_bar.set_fraction(0.0);
    progress_bar.set_text(Some("Starting installation..."));

    // Pulse animation flag
    let installation_done = Arc::new(Mutex::new(false));
    let installation_done_clone = installation_done.clone();

    // Channel for sending progress updates
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<(f64, String)>();

    // UI update: spawn a local idle handler to update the progress bar
    {
        let progress_bar = progress_bar.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Some((fraction, text)) = progress_rx.recv().await {
                let bar = progress_bar.clone();
                idle_add_local(clone!(@strong bar, @strong text => @default-return ControlFlow::Break, move || {
                    bar.set_fraction(fraction);
                    bar.set_text(Some(&text));
                    ControlFlow::Break
                }));
            }
        });
    }

    // Pulse task (UI feedback)
    glib::MainContext::default().spawn_local(clone!(@weak progress_bar => async move {
        let mut pulse_counter = 0;
        while !*installation_done_clone.lock().unwrap() {
            if progress_bar.fraction() < 0.01 {
                progress_bar.pulse();
                pulse_counter += 1;
                if pulse_counter % 5 == 0 {
                    let dots = ".".repeat((pulse_counter / 5) % 4);
                    progress_bar.set_text(Some(&format!("Installing{}", dots)));
                }
            }
            sleep(Duration::from_millis(200)).await;
        }
    }));

    // Prepare command args
    let args = if package_name.is_empty() {
        vec!["pacman", "-Syu", "--noconfirm"]
    } else {
        vec!["pacman", "-S", "--noconfirm", package_name]
    };

    // Spawn pkexec pacman process
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

    // Regex for progress parsing (wrap in Arc for sharing between tasks)
    let re_standard = Arc::new(Regex::new(r"\[#*-*\]\s+(\d+)%").unwrap());
    let re_download = Arc::new(Regex::new(r"downloading.*?\((\d+)%\)").unwrap());

    // Collect error output for reporting
    let error_lines = Arc::new(Mutex::new(Vec::new()));
    let error_lines_clone = error_lines.clone();

    // Read STDOUT
    let tx_stdout = progress_tx.clone();
    let re_standard_stdout = re_standard.clone();
    let re_download_stdout = re_download.clone();
    let stdout_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            let mut sent = false;
            if let Some(caps) = re_standard_stdout.captures(&line) {
                if let Some(percent_match) = caps.get(1) {
                    if let Ok(percent) = percent_match.as_str().parse::<f64>() {
                        let fraction = percent / 100.0;
                        let _ = tx_stdout.send((fraction, format!("{}%", percent)));
                        sent = true;
                    }
                }
            } else if let Some(caps) = re_download_stdout.captures(&line) {
                if let Some(percent_match) = caps.get(1) {
                    if let Ok(percent) = percent_match.as_str().parse::<f64>() {
                        let fraction = percent / 100.0;
                        let _ = tx_stdout.send((fraction, format!("{}%", percent)));
                        sent = true;
                    }
                }
            }
            if !sent {
                // Optionally, send other kinds of updates or log
            }
            eprintln!("STDOUT: {}", line);
        }
    });

    // Read STDERR, collect errors
    let tx_stderr = progress_tx.clone();
    let re_standard_stderr = re_standard.clone();
    let re_download_stderr = re_download.clone();
    let error_lines_clone2 = error_lines.clone();
    let stderr_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            let mut sent = false;
            if let Some(caps) = re_standard_stderr.captures(&line) {
                if let Some(percent_match) = caps.get(1) {
                    if let Ok(percent) = percent_match.as_str().parse::<f64>() {
                        let fraction = percent / 100.0;
                        let _ = tx_stderr.send((fraction, format!("{}%", percent)));
                        sent = true;
                    }
                }
            } else if let Some(caps) = re_download_stderr.captures(&line) {
                if let Some(percent_match) = caps.get(1) {
                    if let Ok(percent) = percent_match.as_str().parse::<f64>() {
                        let fraction = percent / 100.0;
                        let _ = tx_stderr.send((fraction, format!("{}%", percent)));
                        sent = true;
                    }
                }
            }
            // Collect error output for reporting
            error_lines_clone2.lock().unwrap().push(line.clone());
            eprintln!("STDERR: {}", line);
        }
    });

    // Wait for both output tasks and process exit
    let status = child.wait().await.map_err(|e| format!("Installation failed: {}", e))?;
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    // Signal pulse task to stop
    *installation_done.lock().unwrap() = true;

    // Ensure progress bar completion
    progress_bar.set_fraction(1.0);
    progress_bar.set_text(Some("100%"));
    sleep(Duration::from_millis(100)).await;

    // Report errors if process failed
    if status.success() {
        Ok(())
    } else {
        let err_output = error_lines.lock().unwrap().join("\n");
        Err(format!("Installation exited with code: {:?}\n{}", status.code(), err_output))
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
