//! Headless mode: no TUI, just log to stdout.
//!
//! Used with `--headless` flag for running as a background service.

use crate::app::LogEntry;
use crate::bridge;
use crate::config::Config;
use crate::protocol;
use anyhow::Result;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Run the app in headless mode (no TUI, stdout logging).
pub fn run(
    config: Config,
    serial_rx: mpsc::Receiver<String>,
    mut router: bridge::Router,
    ws_incoming_rx: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<()> {
    let mut held_keys: Vec<String> = Vec::new();
    let mut keys_seen_this_tick: Vec<String> = Vec::new();
    let watchdog_interval = Duration::from_millis(config.protocol.watchdog_interval_ms);
    let mut last_watchdog = Instant::now();

    println!("Mio v{} (headless mode)", env!("CARGO_PKG_VERSION"));
    println!("Waiting for serial data...");

    loop {
        // Check for WebSocket incoming messages
        while let Ok(line) = ws_incoming_rx.try_recv() {
            process_line(&line, &mut router, &mut held_keys, &mut keys_seen_this_tick);
        }

        // Watchdog tick
        if last_watchdog.elapsed() >= watchdog_interval {
            let stale: Vec<String> = held_keys
                .iter()
                .filter(|k| !keys_seen_this_tick.contains(k))
                .cloned()
                .collect();
            if !stale.is_empty() {
                router.release_all_keys(&stale);
                held_keys.retain(|k| !stale.contains(k));
                for key in &stale {
                    println!("WATCHDOG: released {}", key);
                }
            }
            keys_seen_this_tick.clear();
            last_watchdog = Instant::now();
        }

        // Check for serial data (non-blocking)
        match serial_rx.recv_timeout(Duration::from_millis(10)) {
            Ok(line) => {
                process_line(&line, &mut router, &mut held_keys, &mut keys_seen_this_tick);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!("Serial port disconnected");
                router.release_all_keys(&held_keys);
                break;
            }
        }
    }

    Ok(())
}

fn process_line(
    line: &str,
    router: &mut bridge::Router,
    held_keys: &mut Vec<String>,
    keys_seen: &mut Vec<String>,
) {
    if let Some(cmd) = protocol::parse(line) {
        match &cmd {
            protocol::Command::KeyDown(key) => {
                if !held_keys.contains(key) {
                    held_keys.push(key.clone());
                }
                keys_seen.push(key.clone());
            }
            protocol::Command::KeyUp(key) => {
                held_keys.retain(|k| k != key);
            }
            _ => {}
        }

        let result = router.dispatch(&cmd);
        let entry = LogEntry::new(line.to_string(), result);
        println!("{} {} -> {}", entry.timestamp, entry.raw_line, entry.result);
    }
}
