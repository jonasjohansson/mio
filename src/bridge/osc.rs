//! OSC (Open Sound Control) output bridge using rosc + UDP.

use crate::config::OscConfig;
use anyhow::{Context, Result};
use rosc::{OscMessage, OscPacket, OscType};
use std::net::UdpSocket;

pub struct OscBridge {
    socket: UdpSocket,
    remote_addr: String,
}

impl OscBridge {
    pub fn new(config: &OscConfig) -> Result<Self> {
        let local_addr = format!("{}:{}", config.local_address, config.local_port);
        let remote_addr = format!("{}:{}", config.remote_address, config.remote_port);

        let socket =
            UdpSocket::bind(&local_addr).with_context(|| format!("Failed to bind OSC socket to {}", local_addr))?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            remote_addr,
        })
    }

    /// Send an OSC message to the configured remote address.
    /// Args are attempted to parse as floats, falling back to strings.
    pub fn send(&self, address: &str, args: &[String]) -> Result<()> {
        let osc_args: Vec<OscType> = args
            .iter()
            .map(|a| {
                // Try float first, then int, then string
                if let Ok(f) = a.parse::<f32>() {
                    OscType::Float(f)
                } else if let Ok(i) = a.parse::<i32>() {
                    OscType::Int(i)
                } else {
                    OscType::String(a.clone())
                }
            })
            .collect();

        let msg = OscPacket::Message(OscMessage {
            addr: address.to_string(),
            args: osc_args,
        });

        let buf = rosc::encoder::encode(&msg).context("Failed to encode OSC message")?;
        self.socket
            .send_to(&buf, &self.remote_addr)
            .context("Failed to send OSC packet")?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn remote_addr(&self) -> &str {
        &self.remote_addr
    }
}
