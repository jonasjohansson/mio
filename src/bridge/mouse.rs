//! Mouse simulation bridge using enigo.

use anyhow::{anyhow, Result};
use enigo::{
    Button, Coordinate,
    Direction::{Click, Press, Release},
    Enigo, Mouse, Settings,
};

pub struct MouseBridge {
    enigo: Enigo,
}

impl MouseBridge {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow!("{}", e))?;
        Ok(Self { enigo })
    }

    pub fn move_to(&mut self, x: i32, y: i32) -> Result<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow!("{}", e))
    }

    pub fn move_relative(&mut self, dx: i32, dy: i32) -> Result<()> {
        self.enigo
            .move_mouse(dx, dy, Coordinate::Rel)
            .map_err(|e| anyhow!("{}", e))
    }

    pub fn click(&mut self, button: &str) -> Result<()> {
        let btn = map_button(button)?;
        self.enigo.button(btn, Click).map_err(|e| anyhow!("{}", e))
    }

    pub fn button_down(&mut self, button: &str) -> Result<()> {
        let btn = map_button(button)?;
        self.enigo.button(btn, Press).map_err(|e| anyhow!("{}", e))
    }

    pub fn button_up(&mut self, button: &str) -> Result<()> {
        let btn = map_button(button)?;
        self.enigo
            .button(btn, Release)
            .map_err(|e| anyhow!("{}", e))
    }

    pub fn scroll(&mut self, x: i32, y: i32) -> Result<()> {
        // Scroll vertically first, then horizontally if needed
        if y != 0 {
            self.enigo.scroll(y, enigo::Axis::Vertical).map_err(|e| anyhow!("{}", e))?;
        }
        if x != 0 {
            self.enigo
                .scroll(x, enigo::Axis::Horizontal)
                .map_err(|e| anyhow!("{}", e))?;
        }
        Ok(())
    }
}

fn map_button(name: &str) -> Result<Button> {
    match name.to_lowercase().as_str() {
        "left" => Ok(Button::Left),
        "right" => Ok(Button::Right),
        "middle" | "center" => Ok(Button::Middle),
        _ => Err(anyhow!("Unknown mouse button: {}", name)),
    }
}
