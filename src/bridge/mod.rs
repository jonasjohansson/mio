//! Bridge router: dispatches parsed Commands to the appropriate bridge.

pub mod keyboard;
pub mod midi;
pub mod mouse;
pub mod osc;
pub mod websocket;

use crate::config::Config;
use crate::protocol::Command;
use anyhow::Result;

/// Central router that holds all enabled bridges and dispatches commands.
pub struct Router {
    pub keyboard: Option<keyboard::KeyboardBridge>,
    pub mouse: Option<mouse::MouseBridge>,
    pub midi: Option<midi::MidiBridge>,
    pub ws_tx: Option<tokio::sync::broadcast::Sender<String>>,
    pub osc: Option<osc::OscBridge>,
}

impl Router {
    /// Create a new router, initializing only the enabled bridges.
    pub fn new(config: &Config) -> Result<Self> {
        let keyboard = if config.keyboard.enabled {
            match keyboard::KeyboardBridge::new() {
                Ok(kb) => Some(kb),
                Err(e) => {
                    eprintln!("[mio] Keyboard bridge unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let mouse = if config.mouse.enabled {
            match mouse::MouseBridge::new() {
                Ok(m) => Some(m),
                Err(e) => {
                    eprintln!("[mio] Mouse bridge unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // MIDI starts disconnected — user connects via TUI or auto_connect
        let midi = if config.midi.enabled {
            Some(midi::MidiBridge::new())
        } else {
            None
        };

        let osc = if config.osc.enabled {
            match osc::OscBridge::new(&config.osc) {
                Ok(o) => Some(o),
                Err(e) => {
                    eprintln!("[mio] OSC bridge unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // WebSocket broadcast channel — the WS server task will subscribe
        let ws_tx = if config.websocket.enabled {
            let (tx, _) = tokio::sync::broadcast::channel(256);
            Some(tx)
        } else {
            None
        };

        Ok(Self {
            keyboard,
            mouse,
            midi,
            ws_tx,
            osc,
        })
    }

    /// Dispatch a command to the appropriate bridge.
    /// Returns a human-readable description of what happened (for the log).
    pub fn dispatch(&mut self, cmd: &Command) -> String {
        match cmd {
            // --- Keyboard ---
            Command::KeyDown(key) => {
                if let Some(kb) = &mut self.keyboard {
                    match kb.key_down(key) {
                        Ok(()) => format!("KEY {} ↓", key),
                        Err(e) => format!("KEY {} ↓ ERROR: {}", key, e),
                    }
                } else {
                    format!("KEY {} ↓ (disabled)", key)
                }
            }
            Command::KeyUp(key) => {
                if let Some(kb) = &mut self.keyboard {
                    match kb.key_up(key) {
                        Ok(()) => format!("KEY {} ↑", key),
                        Err(e) => format!("KEY {} ↑ ERROR: {}", key, e),
                    }
                } else {
                    format!("KEY {} ↑ (disabled)", key)
                }
            }
            Command::KeyTap(key) => {
                if let Some(kb) = &mut self.keyboard {
                    match kb.key_tap(key) {
                        Ok(()) => format!("KEY {} tap", key),
                        Err(e) => format!("KEY {} tap ERROR: {}", key, e),
                    }
                } else {
                    format!("KEY {} tap (disabled)", key)
                }
            }
            Command::KeyType(text) => {
                if let Some(kb) = &mut self.keyboard {
                    match kb.key_type(text) {
                        Ok(()) => format!("KEY type \"{}\"", text),
                        Err(e) => format!("KEY type ERROR: {}", e),
                    }
                } else {
                    format!("KEY type (disabled)")
                }
            }

            // --- Mouse ---
            Command::MouseMove { x, y } => {
                if let Some(m) = &mut self.mouse {
                    match m.move_to(*x, *y) {
                        Ok(()) => format!("MOUSE move ({}, {})", x, y),
                        Err(e) => format!("MOUSE move ERROR: {}", e),
                    }
                } else {
                    format!("MOUSE move (disabled)")
                }
            }
            Command::MouseMoveRel { dx, dy } => {
                if let Some(m) = &mut self.mouse {
                    match m.move_relative(*dx, *dy) {
                        Ok(()) => format!("MOUSE rel ({}, {})", dx, dy),
                        Err(e) => format!("MOUSE rel ERROR: {}", e),
                    }
                } else {
                    format!("MOUSE rel (disabled)")
                }
            }
            Command::MouseClick(button) => {
                if let Some(m) = &mut self.mouse {
                    match m.click(button) {
                        Ok(()) => format!("MOUSE click {}", button),
                        Err(e) => format!("MOUSE click ERROR: {}", e),
                    }
                } else {
                    format!("MOUSE click (disabled)")
                }
            }
            Command::MouseDown(button) => {
                if let Some(m) = &mut self.mouse {
                    match m.button_down(button) {
                        Ok(()) => format!("MOUSE {} ↓", button),
                        Err(e) => format!("MOUSE {} ↓ ERROR: {}", button, e),
                    }
                } else {
                    format!("MOUSE down (disabled)")
                }
            }
            Command::MouseUp(button) => {
                if let Some(m) = &mut self.mouse {
                    match m.button_up(button) {
                        Ok(()) => format!("MOUSE {} ↑", button),
                        Err(e) => format!("MOUSE {} ↑ ERROR: {}", button, e),
                    }
                } else {
                    format!("MOUSE up (disabled)")
                }
            }
            Command::MouseScroll { x, y } => {
                if let Some(m) = &mut self.mouse {
                    match m.scroll(*x, *y) {
                        Ok(()) => format!("MOUSE scroll ({}, {})", x, y),
                        Err(e) => format!("MOUSE scroll ERROR: {}", e),
                    }
                } else {
                    format!("MOUSE scroll (disabled)")
                }
            }

            // --- MIDI ---
            Command::MidiNoteOn { note, velocity, channel } => {
                if let Some(m) = &mut self.midi {
                    match m.note_on(*note, *velocity, *channel) {
                        Ok(()) => format!("MIDI ON note={} vel={} ch={}", note, velocity, channel),
                        Err(e) => format!("MIDI ON ERROR: {}", e),
                    }
                } else {
                    format!("MIDI ON (disabled)")
                }
            }
            Command::MidiNoteOff { note, velocity, channel } => {
                if let Some(m) = &mut self.midi {
                    match m.note_off(*note, *velocity, *channel) {
                        Ok(()) => format!("MIDI OFF note={} ch={}", note, channel),
                        Err(e) => format!("MIDI OFF ERROR: {}", e),
                    }
                } else {
                    format!("MIDI OFF (disabled)")
                }
            }
            Command::MidiCc { controller, value, channel } => {
                if let Some(m) = &mut self.midi {
                    match m.cc(*controller, *value, *channel) {
                        Ok(()) => format!("MIDI CC {}={} ch={}", controller, value, channel),
                        Err(e) => format!("MIDI CC ERROR: {}", e),
                    }
                } else {
                    format!("MIDI CC (disabled)")
                }
            }
            Command::MidiRaw { bytes } => {
                if let Some(m) = &mut self.midi {
                    match m.raw(bytes) {
                        Ok(()) => format!("MIDI raw [{}, {}, {}]", bytes[0], bytes[1], bytes[2]),
                        Err(e) => format!("MIDI raw ERROR: {}", e),
                    }
                } else {
                    format!("MIDI raw (disabled)")
                }
            }

            // --- WebSocket ---
            Command::WsBroadcast { id, value } => {
                if let Some(tx) = &self.ws_tx {
                    let json = format!("{{\"id\":\"{}\",\"value\":{}}}", id, value);
                    let count = tx.receiver_count();
                    let _ = tx.send(json);
                    format!("WS broadcast {}={} ({} clients)", id, value, count)
                } else {
                    format!("WS broadcast (disabled)")
                }
            }
            Command::WsRaw(payload) => {
                if let Some(tx) = &self.ws_tx {
                    let count = tx.receiver_count();
                    let _ = tx.send(payload.clone());
                    format!("WS raw ({} clients)", count)
                } else {
                    format!("WS raw (disabled)")
                }
            }

            // --- OSC ---
            Command::OscMessage { address, args } => {
                if let Some(o) = &self.osc {
                    match o.send(address, args) {
                        Ok(()) => format!("OSC {} [{}]", address, args.join(", ")),
                        Err(e) => format!("OSC ERROR: {}", e),
                    }
                } else {
                    format!("OSC (disabled)")
                }
            }
        }
    }

    /// Release all currently held keys. Called on disconnect / shutdown.
    pub fn release_all_keys(&mut self, held_keys: &[String]) {
        if let Some(kb) = &mut self.keyboard {
            for key in held_keys {
                let _ = kb.key_up(key);
            }
        }
    }
}
