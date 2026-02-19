//! App struct: unified event loop, state management, key watchdog.
//!
//! The main thread runs: TUI rendering, event processing, and enigo
//! (macOS requires CGEvent calls on the main thread).

use crate::bridge::{self, websocket};
use crate::config::Config;
use crate::protocol;
use crate::serial;
use crate::tui::{self, event::TuiAction, layout, widgets::Popup};
use anyhow::Result;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// A single log entry displayed in the TUI.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub raw_line: String,
    pub result: String,
}

impl LogEntry {
    pub fn new(raw_line: String, result: String) -> Self {
        Self {
            timestamp: now_hms(),
            raw_line,
            result,
        }
    }
}

/// Format current local time as HH:MM:SS using only std.
pub fn now_hms() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Get local UTC offset on macOS/Linux via libc (good enough for a timestamp)
    let local_secs = secs as i64 + local_utc_offset_secs();
    let day_secs = local_secs.rem_euclid(86400);
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Get the local UTC offset in seconds using libc's localtime.
fn local_utc_offset_secs() -> i64 {
    #[cfg(unix)]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        // SAFETY: localtime_r is thread-safe and we provide a valid pointer
        unsafe {
            let mut tm: libc::tm = std::mem::zeroed();
            libc::localtime_r(&timestamp as *const i64, &mut tm);
            tm.tm_gmtoff
        }
    }
    #[cfg(not(unix))]
    {
        0 // Fall back to UTC on non-unix
    }
}

/// The full application state, passed to the TUI renderer each frame.
pub struct AppState {
    pub serial_connected: bool,
    pub serial_port_name: Option<String>,
    pub baud_rate: u32,
    pub midi_port_name: Option<String>,
    pub ws_enabled: bool,
    pub ws_port: u16,
    pub ws_client_count: usize,
    pub osc_enabled: bool,
    pub osc_remote: String,
    pub log_lines: Vec<LogEntry>,
    pub max_log_lines: usize,
    pub show_timestamps: bool,
    pub scroll_offset: usize,
    pub active_popup: Option<Popup>,
}

impl AppState {
    pub fn from_config(config: &Config) -> Self {
        Self {
            serial_connected: false,
            serial_port_name: None,
            baud_rate: config.serial.baud_rate,
            midi_port_name: None,
            ws_enabled: config.websocket.enabled,
            ws_port: config.websocket.port,
            ws_client_count: 0,
            osc_enabled: config.osc.enabled,
            osc_remote: format!("{}:{}", config.osc.remote_address, config.osc.remote_port),
            log_lines: Vec::new(),
            max_log_lines: config.tui.max_log_lines,
            show_timestamps: config.tui.show_timestamps,
            scroll_offset: 0,
            active_popup: None,
        }
    }

    fn push_log(&mut self, entry: LogEntry) {
        self.log_lines.push(entry);
        if self.log_lines.len() > self.max_log_lines {
            self.log_lines.remove(0);
        }
    }

    fn push_info(&mut self, message: String) {
        self.log_lines.push(LogEntry {
            timestamp: now_hms(),
            raw_line: String::new(),
            result: message,
        });
    }
}

/// Run the full TUI application.
pub fn run(
    config: Config,
    mut router: bridge::Router,
    ws_client_count: Option<websocket::ClientCount>,
    mut ws_incoming_rx: tokio::sync::mpsc::Receiver<String>,
) -> Result<()> {
    let mut terminal = tui::init()?;
    let mut state = AppState::from_config(&config);

    // Serial reader channel — initially no reader
    let (serial_tx, serial_rx) = mpsc::channel::<String>();
    let mut _serial_handle: Option<serial::SerialHandle> = None;

    // Held keys for the watchdog
    let mut held_keys: Vec<String> = Vec::new();
    // Track which keys were refreshed this tick
    let mut keys_seen_this_tick: Vec<String> = Vec::new();

    let watchdog_interval = Duration::from_millis(config.protocol.watchdog_interval_ms);
    let mut last_watchdog = Instant::now();

    state.push_info("Mio started. Press [c] to connect serial, [?] for help.".into());

    loop {
        // --- Render ---
        if let Some(count) = &ws_client_count {
            state.ws_client_count = count.load(Ordering::Relaxed);
        }
        terminal.draw(|frame| layout::render(frame, &state))?;

        // --- Handle TUI input (with short timeout to keep the loop responsive) ---
        if let Some(action) = tui::event::poll(Duration::from_millis(16)) {
            match (&state.active_popup, action) {
                // Popup is active — handle popup-specific keys
                (Some(Popup::Help), TuiAction::DismissPopup | TuiAction::ShowHelp) => {
                    state.active_popup = None;
                }
                (Some(Popup::PortSelect { ports, selected }), TuiAction::ScrollDown) => {
                    let new_sel = (*selected + 1).min(ports.len().saturating_sub(1));
                    let ports = ports.clone();
                    state.active_popup = Some(Popup::PortSelect { ports, selected: new_sel });
                }
                (Some(Popup::PortSelect { ports, selected }), TuiAction::ScrollUp) => {
                    let new_sel = selected.saturating_sub(1);
                    let ports = ports.clone();
                    state.active_popup = Some(Popup::PortSelect { ports, selected: new_sel });
                }
                (Some(Popup::PortSelect { ports, selected }), TuiAction::Confirm) => {
                    if let Some(port) = ports.get(*selected) {
                        let port_name = port.name.clone();
                        match serial::spawn_reader(&port_name, state.baud_rate, serial_tx.clone()) {
                            Ok(handle) => {
                                _serial_handle = Some(handle);
                                state.serial_connected = true;
                                state.serial_port_name = Some(port_name.clone());
                                state.push_info(format!("Connected to {}", port_name));
                            }
                            Err(e) => {
                                state.push_info(format!("Failed to connect: {}", e));
                            }
                        }
                    }
                    state.active_popup = None;
                }
                (Some(Popup::MidiSelect { ports, selected }), TuiAction::ScrollDown) => {
                    let new_sel = (*selected + 1).min(ports.len().saturating_sub(1));
                    let ports = ports.clone();
                    state.active_popup = Some(Popup::MidiSelect { ports, selected: new_sel });
                }
                (Some(Popup::MidiSelect { ports, selected }), TuiAction::ScrollUp) => {
                    let new_sel = selected.saturating_sub(1);
                    let ports = ports.clone();
                    state.active_popup = Some(Popup::MidiSelect { ports, selected: new_sel });
                }
                (Some(Popup::MidiSelect { ports, selected }), TuiAction::Confirm) => {
                    if let Some(midi_bridge) = &mut router.midi {
                        if let Some(port) = ports.get(*selected) {
                            match midi_bridge.connect(port.index) {
                                Ok(name) => {
                                    state.midi_port_name = Some(name.clone());
                                    state.push_info(format!("MIDI connected: {}", name));
                                }
                                Err(e) => {
                                    state.push_info(format!("MIDI connect failed: {}", e));
                                }
                            }
                        }
                    }
                    state.active_popup = None;
                }
                (Some(_), TuiAction::DismissPopup) => {
                    state.active_popup = None;
                }
                (Some(_), TuiAction::Quit) => {
                    break;
                }

                // No popup — normal key handling
                (None, TuiAction::Quit) => break,
                (None, TuiAction::ToggleConnect) => {
                    if state.serial_connected {
                        // Disconnect
                        _serial_handle = None;
                        state.serial_connected = false;
                        let port = state.serial_port_name.take().unwrap_or_default();
                        router.release_all_keys(&held_keys);
                        held_keys.clear();
                        state.push_info(format!("Disconnected from {}", port));
                    } else {
                        // Show port selection popup
                        match serial::list_ports() {
                            Ok(ports) if !ports.is_empty() => {
                                state.active_popup = Some(Popup::PortSelect { ports, selected: 0 });
                            }
                            Ok(_) => {
                                state.push_info("No serial ports found".into());
                            }
                            Err(e) => {
                                state.push_info(format!("Error listing ports: {}", e));
                            }
                        }
                    }
                }
                (None, TuiAction::ToggleMidi) => {
                    if state.midi_port_name.is_some() {
                        // Disconnect MIDI
                        if let Some(midi_bridge) = &mut router.midi {
                            midi_bridge.disconnect();
                        }
                        let name = state.midi_port_name.take().unwrap_or_default();
                        state.push_info(format!("MIDI disconnected: {}", name));
                    } else {
                        // Show MIDI port selection popup
                        if let Some(midi_bridge) = &router.midi {
                            let ports = midi_bridge.list_ports();
                            if ports.is_empty() {
                                state.push_info("No MIDI output ports found".into());
                            } else {
                                state.active_popup = Some(Popup::MidiSelect { ports, selected: 0 });
                            }
                        } else {
                            state.push_info("MIDI bridge is disabled".into());
                        }
                    }
                }
                (None, TuiAction::ShowHelp) => {
                    state.active_popup = Some(Popup::Help);
                }
                (None, TuiAction::ScrollUp) => {
                    state.scroll_offset = state.scroll_offset.saturating_add(1);
                }
                (None, TuiAction::ScrollDown) => {
                    state.scroll_offset = state.scroll_offset.saturating_sub(1);
                }
                _ => {}
            }
        }

        // --- Process serial data ---
        while let Ok(line) = serial_rx.try_recv() {
            if let Some(cmd) = protocol::parse(&line) {
                // Track held keys for watchdog
                match &cmd {
                    protocol::Command::KeyDown(key) => {
                        if !held_keys.contains(key) {
                            held_keys.push(key.clone());
                        }
                        keys_seen_this_tick.push(key.clone());
                    }
                    protocol::Command::KeyUp(key) => {
                        held_keys.retain(|k| k != key);
                    }
                    _ => {}
                }

                let result = router.dispatch(&cmd);
                state.push_log(LogEntry::new(line, result));
                state.scroll_offset = 0;
            }
        }

        // --- Process WebSocket incoming messages ---
        while let Ok(line) = ws_incoming_rx.try_recv() {
            if let Some(cmd) = protocol::parse(&line) {
                let result = router.dispatch(&cmd);
                state.push_log(LogEntry::new(format!("[ws] {}", line), result));
                state.scroll_offset = 0;
            }
        }

        // --- Watchdog tick ---
        if last_watchdog.elapsed() >= watchdog_interval {
            let stale_keys: Vec<String> = held_keys
                .iter()
                .filter(|k| !keys_seen_this_tick.contains(k))
                .cloned()
                .collect();

            for key in &stale_keys {
                if let Some(kb) = &mut router.keyboard {
                    let _ = kb.key_up(key);
                }
                held_keys.retain(|k| k != key);
                state.push_info(format!("WATCHDOG: released {}", key));
            }

            keys_seen_this_tick.clear();
            last_watchdog = Instant::now();
        }
    }

    // --- Graceful shutdown ---
    router.release_all_keys(&held_keys);
    tui::restore()?;

    Ok(())
}
