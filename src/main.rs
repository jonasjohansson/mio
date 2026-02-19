//! Mio — Serial-to-Everything Bridge
//!
//! A single binary that bridges serial communication to keyboard/mouse input,
//! MIDI output, WebSocket broadcasts, and OSC messages.
//!
//! Usage: mio [OPTIONS]
//! See `mio --help` for details.

mod app;
mod bridge;
mod config;
mod headless;
mod protocol;
mod serial;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Mio — Serial-to-Everything Bridge
#[derive(Parser, Debug)]
#[command(name = "mio", version, about = "Serial-to-everything bridge: keyboard, mouse, MIDI, WebSocket, OSC")]
struct Cli {
    /// Path to config file (default: ./mio.toml or ~/.config/mio/mio.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// List available serial ports and exit
    #[arg(long)]
    list_ports: bool,

    /// List available MIDI output ports and exit
    #[arg(long)]
    list_midi: bool,

    /// Run in headless mode (no TUI, log to stdout)
    #[arg(long)]
    headless: bool,

    /// Serial port to connect to immediately
    #[arg(short, long)]
    port: Option<String>,

    /// Baud rate (overrides config)
    #[arg(short, long)]
    baud: Option<u32>,

    /// WebSocket server port (overrides config)
    #[arg(long)]
    ws_port: Option<u16>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // --- List ports mode ---
    if cli.list_ports {
        let ports = serial::list_ports()?;
        if ports.is_empty() {
            println!("No serial ports found.");
        } else {
            println!("Available serial ports:");
            for port in &ports {
                println!("  {} ({})", port.name, port.port_type);
            }
        }
        return Ok(());
    }

    // --- List MIDI mode ---
    if cli.list_midi {
        let ports = bridge::midi::list_midi_ports();
        if ports.is_empty() {
            println!("No MIDI output ports found.");
        } else {
            println!("Available MIDI output ports:");
            for port in &ports {
                println!("  [{}] {}", port.index, port.name);
            }
        }
        return Ok(());
    }

    // --- Load config ---
    let mut config = config::load(cli.config.as_deref())?;

    // Apply CLI overrides
    if let Some(baud) = cli.baud {
        config.serial.baud_rate = baud;
    }
    if let Some(ws_port) = cli.ws_port {
        config.websocket.port = ws_port;
    }

    // --- Build the tokio runtime (on a background thread) ---
    let runtime = tokio::runtime::Runtime::new()?;

    // --- Initialize the bridge router ---
    let router = runtime.block_on(async { bridge::Router::new(&config) })?;

    // --- Start WebSocket server if enabled ---
    let (ws_client_count, ws_incoming_rx) = if config.websocket.enabled {
        if let Some(ws_tx) = &router.ws_tx {
            let (incoming_tx, incoming_rx) = tokio::sync::mpsc::channel(256);
            let ws_tx_clone = ws_tx.clone();
            let host = config.websocket.host.clone();
            let port = config.websocket.port;

            let (count, _handle) = runtime.block_on(async {
                bridge::websocket::start_server(&host, port, ws_tx_clone, incoming_tx)
                    .await
                    .expect("Failed to start WebSocket server")
            });

            (Some(count), incoming_rx)
        } else {
            let (_tx, rx) = tokio::sync::mpsc::channel(1);
            (None, rx)
        }
    } else {
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        (None, rx)
    };

    // --- Run the app ---
    if cli.headless {
        let (serial_tx, serial_rx) = std::sync::mpsc::channel();
        let mut ws_incoming_rx = ws_incoming_rx;

        let _serial_handle = if let Some(port_name) = &cli.port {
            Some(serial::spawn_reader(port_name, config.serial.baud_rate, serial_tx)?)
        } else {
            println!("No --port specified. Use --port <name> in headless mode.");
            println!("Available ports:");
            for port in serial::list_ports()? {
                println!("  {} ({})", port.name, port.port_type);
            }
            return Ok(());
        };

        headless::run(config, serial_rx, router, &mut ws_incoming_rx)?;
    } else {
        app::run(config, router, ws_client_count, ws_incoming_rx)?;
    }

    Ok(())
}
