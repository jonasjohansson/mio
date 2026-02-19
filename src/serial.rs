//! Serial port discovery and reading.
//!
//! The serial reader runs in a dedicated std::thread (blocking I/O)
//! and sends complete lines to the main event loop via mpsc channel.

use anyhow::{Context, Result};
use std::io::BufRead;
use std::sync::mpsc;
use std::time::Duration;

/// Information about an available serial port.
#[derive(Debug, Clone)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
}

/// List all available serial ports.
pub fn list_ports() -> Result<Vec<PortInfo>> {
    let ports = serialport::available_ports().context("Failed to list serial ports")?;
    Ok(ports
        .into_iter()
        .map(|p| {
            let port_type = match &p.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    let product = info.product.as_deref().unwrap_or("Unknown");
                    let manufacturer = info.manufacturer.as_deref().unwrap_or("");
                    if manufacturer.is_empty() {
                        format!("USB ({})", product)
                    } else {
                        format!("USB ({} - {})", manufacturer, product)
                    }
                }
                serialport::SerialPortType::BluetoothPort => "Bluetooth".into(),
                serialport::SerialPortType::PciPort => "PCI".into(),
                serialport::SerialPortType::Unknown => "Unknown".into(),
            };
            PortInfo {
                name: p.port_name,
                port_type,
            }
        })
        .collect())
}

/// Spawn a blocking reader thread for the given serial port.
/// Returns a receiver that yields complete lines (without the newline).
/// Also returns a `stop` sender — drop it or send () to signal shutdown.
pub fn spawn_reader(
    port_name: &str,
    baud_rate: u32,
    line_tx: mpsc::Sender<String>,
) -> Result<SerialHandle> {
    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
        .with_context(|| format!("Failed to open serial port: {}", port_name))?;

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let port_name_owned = port_name.to_string();

    let handle = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(port);
        let mut line_buf = String::new();

        loop {
            // Check if we've been asked to stop
            if stop_rx.try_recv().is_ok() {
                break;
            }

            line_buf.clear();
            match reader.read_line(&mut line_buf) {
                Ok(0) => {
                    // EOF — port closed
                    break;
                }
                Ok(_) => {
                    let trimmed = line_buf.trim().to_string();
                    if !trimmed.is_empty() {
                        if line_tx.send(trimmed).is_err() {
                            // Receiver dropped, time to quit
                            break;
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Normal timeout — just loop again
                    continue;
                }
                Err(_) => {
                    // Port error (disconnected, etc.)
                    break;
                }
            }
        }

        eprintln!("[mio] serial reader exiting for {}", port_name_owned);
    });

    Ok(SerialHandle {
        _stop_tx: stop_tx,
        _thread: handle,
    })
}

/// Handle for a running serial reader. Dropping this signals the thread to stop.
pub struct SerialHandle {
    _stop_tx: mpsc::Sender<()>,
    _thread: std::thread::JoinHandle<()>,
}
