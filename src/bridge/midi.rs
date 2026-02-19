//! MIDI output bridge using midir.

use anyhow::{anyhow, Result};
use midir::{MidiOutput, MidiOutputConnection};

pub struct MidiBridge {
    output: MidiOutput,
    connection: Option<MidiOutputConnection>,
    connected_port_name: Option<String>,
}

/// Information about an available MIDI output port.
#[derive(Debug, Clone)]
pub struct MidiPortInfo {
    pub index: usize,
    pub name: String,
}

impl MidiBridge {
    pub fn new() -> Self {
        let output = MidiOutput::new("Mio").expect("Failed to create MIDI output");
        Self {
            output,
            connection: None,
            connected_port_name: None,
        }
    }

    /// List available MIDI output ports.
    pub fn list_ports(&self) -> Vec<MidiPortInfo> {
        let ports = self.output.ports();
        ports
            .iter()
            .enumerate()
            .map(|(i, p)| MidiPortInfo {
                index: i,
                name: self.output.port_name(p).unwrap_or_else(|_| "Unknown".into()),
            })
            .collect()
    }

    /// Connect to a MIDI output port by index.
    pub fn connect(&mut self, port_index: usize) -> Result<String> {
        // Close existing connection first
        self.disconnect();

        // We need to recreate the MidiOutput because `connect` consumes it.
        // midir's API requires us to create a new MidiOutput each time.
        let output = MidiOutput::new("Mio").map_err(|e| anyhow!("{}", e))?;
        let ports = output.ports();
        let port = ports
            .get(port_index)
            .ok_or_else(|| anyhow!("MIDI port index {} not found", port_index))?;
        let port_name = output
            .port_name(port)
            .unwrap_or_else(|_| "Unknown".into());

        let conn = output
            .connect(port, "mio-output")
            .map_err(|e| anyhow!("Failed to connect to MIDI port: {}", e))?;

        self.connection = Some(conn);
        self.connected_port_name = Some(port_name.clone());

        // Recreate the MidiOutput for future port listing
        self.output = MidiOutput::new("Mio").map_err(|e| anyhow!("{}", e))?;

        Ok(port_name)
    }

    /// Disconnect from the current MIDI port.
    pub fn disconnect(&mut self) {
        if let Some(conn) = self.connection.take() {
            conn.close();
        }
        self.connected_port_name = None;
    }

    #[allow(dead_code)]
    pub fn connected_port(&self) -> Option<&str> {
        self.connected_port_name.as_deref()
    }

    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn note_on(&mut self, note: u8, velocity: u8, channel: u8) -> Result<()> {
        let status = 0x90 | (channel & 0x0F);
        self.send_bytes(&[status, note & 0x7F, velocity & 0x7F])
    }

    pub fn note_off(&mut self, note: u8, velocity: u8, channel: u8) -> Result<()> {
        let status = 0x80 | (channel & 0x0F);
        self.send_bytes(&[status, note & 0x7F, velocity & 0x7F])
    }

    pub fn cc(&mut self, controller: u8, value: u8, channel: u8) -> Result<()> {
        let status = 0xB0 | (channel & 0x0F);
        self.send_bytes(&[status, controller & 0x7F, value & 0x7F])
    }

    pub fn raw(&mut self, bytes: &[u8; 3]) -> Result<()> {
        self.send_bytes(bytes)
    }

    fn send_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        match &mut self.connection {
            Some(conn) => conn.send(bytes).map_err(|e| anyhow!("MIDI send error: {}", e)),
            None => Err(anyhow!("MIDI not connected")),
        }
    }
}

/// Standalone function to list MIDI ports (for --list-midi CLI flag).
pub fn list_midi_ports() -> Vec<MidiPortInfo> {
    let bridge = MidiBridge::new();
    bridge.list_ports()
}
