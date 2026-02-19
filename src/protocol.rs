//! Protocol parser: turns a serial line into a typed Command.
//!
//! Every message is a single line (`\n`-terminated).
//! Format: `PREFIX:SUBCOMMAND,arg1,arg2,...`
//!
//! Examples:
//!   key:tap,a        -> KeyTap("a")
//!   mouse:move,100,200 -> MouseMove(100, 200)
//!   midi:note_on,60,127,0 -> MidiNoteOn { note: 60, velocity: 127, channel: 0 }
//!   ws:temperature,23.5 -> WsBroadcast { id: "temperature", value: "23.5" }
//!   osc:/sensor/temp,23.5 -> OscMessage { address: "/sensor/temp", args: ["23.5"] }

/// A parsed command from the serial protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // --- Keyboard ---
    KeyDown(String),
    KeyUp(String),
    KeyTap(String),
    KeyType(String),

    // --- Mouse ---
    MouseMove { x: i32, y: i32 },
    MouseMoveRel { dx: i32, dy: i32 },
    MouseClick(String),
    MouseDown(String),
    MouseUp(String),
    MouseScroll { x: i32, y: i32 },

    // --- MIDI ---
    MidiNoteOn { note: u8, velocity: u8, channel: u8 },
    MidiNoteOff { note: u8, velocity: u8, channel: u8 },
    MidiCc { controller: u8, value: u8, channel: u8 },
    MidiRaw { bytes: [u8; 3] },

    // --- WebSocket ---
    WsBroadcast { id: String, value: String },
    WsRaw(String),

    // --- OSC ---
    OscMessage { address: String, args: Vec<String> },
}

/// Parse a single line from the serial port into a Command.
/// Returns None if the line is empty or not recognized.
pub fn parse(line: &str) -> Option<Command> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Split on the first ':' to get the prefix
    let colon_pos = line.find(':')?;
    let prefix = &line[..colon_pos];
    let rest = &line[colon_pos + 1..];

    match prefix {
        "key" => parse_key(rest),
        "mouse" => parse_mouse(rest),
        "midi" => parse_midi(rest),
        "ws" => parse_ws(rest),
        "osc" => parse_osc(rest),
        _ => None,
    }
}

/// Parse key commands: down, up, tap, type
fn parse_key(rest: &str) -> Option<Command> {
    let (sub, arg) = split_sub_and_args(rest);
    let key = arg.first().map(|s| s.to_string())?;

    match sub {
        "down" => Some(Command::KeyDown(key)),
        "up" => Some(Command::KeyUp(key)),
        "tap" => Some(Command::KeyTap(key)),
        "type" => {
            // For type, rejoin all args in case the text contained commas
            let text = rest.strip_prefix("type,")?.to_string();
            Some(Command::KeyType(text))
        }
        _ => None,
    }
}

/// Parse mouse commands: move, move_rel, click, down, up, scroll
fn parse_mouse(rest: &str) -> Option<Command> {
    let (sub, args) = split_sub_and_args(rest);

    match sub {
        "move" => {
            let x = args.first()?.parse().ok()?;
            let y = args.get(1)?.parse().ok()?;
            Some(Command::MouseMove { x, y })
        }
        "move_rel" => {
            let dx = args.first()?.parse().ok()?;
            let dy = args.get(1)?.parse().ok()?;
            Some(Command::MouseMoveRel { dx, dy })
        }
        "click" => {
            let button = args.first().map(|s| s.to_string()).unwrap_or_else(|| "left".into());
            Some(Command::MouseClick(button))
        }
        "down" => {
            let button = args.first().map(|s| s.to_string()).unwrap_or_else(|| "left".into());
            Some(Command::MouseDown(button))
        }
        "up" => {
            let button = args.first().map(|s| s.to_string()).unwrap_or_else(|| "left".into());
            Some(Command::MouseUp(button))
        }
        "scroll" => {
            let x = args.first()?.parse().ok()?;
            let y = args.get(1)?.parse().ok()?;
            Some(Command::MouseScroll { x, y })
        }
        _ => None,
    }
}

/// Parse MIDI commands: note_on, note_off, cc, raw
fn parse_midi(rest: &str) -> Option<Command> {
    let (sub, args) = split_sub_and_args(rest);

    match sub {
        "note_on" => {
            let note = args.first()?.parse().ok()?;
            let velocity = args.get(1)?.parse().ok()?;
            let channel = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            Some(Command::MidiNoteOn { note, velocity, channel })
        }
        "note_off" => {
            let note = args.first()?.parse().ok()?;
            let velocity = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let channel = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            Some(Command::MidiNoteOff { note, velocity, channel })
        }
        "cc" => {
            let controller = args.first()?.parse().ok()?;
            let value = args.get(1)?.parse().ok()?;
            let channel = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            Some(Command::MidiCc { controller, value, channel })
        }
        "raw" => {
            let b0 = args.first()?.parse().ok()?;
            let b1 = args.get(1)?.parse().ok()?;
            let b2 = args.get(2)?.parse().ok()?;
            Some(Command::MidiRaw { bytes: [b0, b1, b2] })
        }
        _ => None,
    }
}

/// Parse WebSocket commands: named broadcast or raw
fn parse_ws(rest: &str) -> Option<Command> {
    // ws:raw,{...} — everything after "raw," is the raw payload
    if let Some(payload) = rest.strip_prefix("raw,") {
        return Some(Command::WsRaw(payload.to_string()));
    }

    // ws:temperature,23.5 — id,value
    let comma_pos = rest.find(',')?;
    let id = rest[..comma_pos].to_string();
    let value = rest[comma_pos + 1..].to_string();
    Some(Command::WsBroadcast { id, value })
}

/// Parse OSC commands: osc:/address,arg1,arg2,...
fn parse_osc(rest: &str) -> Option<Command> {
    // The rest starts with the OSC address (e.g., /sensor/temp,23.5)
    let comma_pos = rest.find(',');

    match comma_pos {
        Some(pos) => {
            let address = rest[..pos].to_string();
            let args_str = &rest[pos + 1..];
            let args: Vec<String> = args_str.split(',').map(|s| s.to_string()).collect();
            Some(Command::OscMessage { address, args })
        }
        None => {
            // No args, just an address trigger
            Some(Command::OscMessage {
                address: rest.to_string(),
                args: vec![],
            })
        }
    }
}

/// Split "subcommand,arg1,arg2" into ("subcommand", ["arg1", "arg2"])
fn split_sub_and_args(rest: &str) -> (&str, Vec<&str>) {
    let mut parts = rest.splitn(2, ',');
    let sub = parts.next().unwrap_or("");
    let args: Vec<&str> = match parts.next() {
        Some(arg_str) => arg_str.split(',').collect(),
        None => vec![],
    };
    (sub, args)
}
