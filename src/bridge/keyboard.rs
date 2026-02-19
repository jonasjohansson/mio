//! Keyboard simulation bridge using enigo.
//!
//! Must run on the main thread on macOS (CGEvent requirement).

use anyhow::{anyhow, Result};
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

pub struct KeyboardBridge {
    enigo: Enigo,
}

impl KeyboardBridge {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow!("{}", e))?;
        Ok(Self { enigo })
    }

    pub fn key_down(&mut self, key: &str) -> Result<()> {
        let k = map_key(key)?;
        self.enigo.key(k, Press).map_err(|e| anyhow!("{}", e))
    }

    pub fn key_up(&mut self, key: &str) -> Result<()> {
        let k = map_key(key)?;
        self.enigo.key(k, Release).map_err(|e| anyhow!("{}", e))
    }

    pub fn key_tap(&mut self, key: &str) -> Result<()> {
        let k = map_key(key)?;
        self.enigo.key(k, Click).map_err(|e| anyhow!("{}", e))
    }

    pub fn key_type(&mut self, text: &str) -> Result<()> {
        self.enigo.text(text).map_err(|e| anyhow!("{}", e))
    }
}

/// Map a key name string to an enigo Key.
fn map_key(name: &str) -> Result<Key> {
    // Single character — use Unicode key
    let lower = name.to_lowercase();
    if lower.len() == 1 {
        let ch = lower.chars().next().unwrap();
        return Ok(Key::Unicode(ch));
    }

    // Named keys
    match lower.as_str() {
        "space" => Ok(Key::Unicode(' ')),
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "delete" => Ok(Key::Delete),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        "shift" => Ok(Key::Shift),
        "control" | "ctrl" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "command" | "cmd" | "meta" | "super" => Ok(Key::Meta),
        "capslock" => Ok(Key::CapsLock),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        _ => Err(anyhow!("Unknown key: {}", name)),
    }
}
