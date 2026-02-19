//! Configuration file loading and defaults.
//!
//! Search order: --config <path> > ./mio.toml > ~/.config/mio/mio.toml > defaults

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level configuration, mirrors the `mio.toml` file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub serial: SerialConfig,
    pub protocol: ProtocolConfig,
    pub keyboard: KeyboardConfig,
    pub mouse: MouseConfig,
    pub midi: MidiConfig,
    pub websocket: WebSocketConfig,
    pub osc: OscConfig,
    pub tui: TuiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProtocolConfig {
    pub watchdog_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MidiConfig {
    pub enabled: bool,
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OscConfig {
    pub enabled: bool,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    pub show_timestamps: bool,
    pub max_log_lines: usize,
}

// --- Defaults ---

impl Default for Config {
    fn default() -> Self {
        Self {
            serial: SerialConfig::default(),
            protocol: ProtocolConfig::default(),
            keyboard: KeyboardConfig::default(),
            mouse: MouseConfig::default(),
            midi: MidiConfig::default(),
            websocket: WebSocketConfig::default(),
            osc: OscConfig::default(),
            tui: TuiConfig::default(),
        }
    }
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baud_rate: 9600,
            auto_connect: false,
        }
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            watchdog_interval_ms: 100,
        }
    }
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for MidiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_connect: false,
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8080,
            host: "0.0.0.0".into(),
        }
    }
}

impl Default for OscConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            local_address: "0.0.0.0".into(),
            local_port: 7000,
            remote_address: "127.0.0.1".into(),
            remote_port: 7001,
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            show_timestamps: true,
            max_log_lines: 1000,
        }
    }
}

/// Find the config file path using the search order:
/// 1. Explicit --config path
/// 2. ./mio.toml
/// 3. ~/.config/mio/mio.toml
/// Returns None if no config file is found (defaults will be used).
pub fn find_config_path(explicit: Option<&Path>) -> Option<PathBuf> {
    // 1. Explicit path from CLI
    if let Some(path) = explicit {
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    // 2. Current directory
    let local = PathBuf::from("mio.toml");
    if local.exists() {
        return Some(local);
    }

    // 3. XDG / home config directory
    if let Some(config_dir) = dirs::config_dir() {
        let global = config_dir.join("mio").join("mio.toml");
        if global.exists() {
            return Some(global);
        }
    }

    None
}

/// Load the config from a file path, falling back to defaults.
pub fn load(explicit_path: Option<&Path>) -> Result<Config> {
    match find_config_path(explicit_path) {
        Some(path) => {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
            Ok(config)
        }
        None => Ok(Config::default()),
    }
}
