//! Tests for the serial protocol parser.

use mio_bridge::protocol::{self, Command};

// --- Keyboard ---

#[test]
fn test_key_down() {
    assert_eq!(
        protocol::parse("key:down,space"),
        Some(Command::KeyDown("space".into()))
    );
}

#[test]
fn test_key_up() {
    assert_eq!(
        protocol::parse("key:up,space"),
        Some(Command::KeyUp("space".into()))
    );
}

#[test]
fn test_key_tap() {
    assert_eq!(
        protocol::parse("key:tap,a"),
        Some(Command::KeyTap("a".into()))
    );
}

#[test]
fn test_key_type() {
    assert_eq!(
        protocol::parse("key:type,hello"),
        Some(Command::KeyType("hello".into()))
    );
}

#[test]
fn test_key_type_with_commas() {
    assert_eq!(
        protocol::parse("key:type,hello, world"),
        Some(Command::KeyType("hello, world".into()))
    );
}

// --- Mouse ---

#[test]
fn test_mouse_move() {
    assert_eq!(
        protocol::parse("mouse:move,100,200"),
        Some(Command::MouseMove { x: 100, y: 200 })
    );
}

#[test]
fn test_mouse_move_rel() {
    assert_eq!(
        protocol::parse("mouse:move_rel,10,-5"),
        Some(Command::MouseMoveRel { dx: 10, dy: -5 })
    );
}

#[test]
fn test_mouse_click() {
    assert_eq!(
        protocol::parse("mouse:click,left"),
        Some(Command::MouseClick("left".into()))
    );
}

#[test]
fn test_mouse_down() {
    assert_eq!(
        protocol::parse("mouse:down,left"),
        Some(Command::MouseDown("left".into()))
    );
}

#[test]
fn test_mouse_up() {
    assert_eq!(
        protocol::parse("mouse:up,left"),
        Some(Command::MouseUp("left".into()))
    );
}

#[test]
fn test_mouse_scroll() {
    assert_eq!(
        protocol::parse("mouse:scroll,0,5"),
        Some(Command::MouseScroll { x: 0, y: 5 })
    );
}

// --- MIDI ---

#[test]
fn test_midi_note_on() {
    assert_eq!(
        protocol::parse("midi:note_on,60,127,0"),
        Some(Command::MidiNoteOn {
            note: 60,
            velocity: 127,
            channel: 0
        })
    );
}

#[test]
fn test_midi_note_off() {
    assert_eq!(
        protocol::parse("midi:note_off,60,0,0"),
        Some(Command::MidiNoteOff {
            note: 60,
            velocity: 0,
            channel: 0
        })
    );
}

#[test]
fn test_midi_cc() {
    assert_eq!(
        protocol::parse("midi:cc,44,127,0"),
        Some(Command::MidiCc {
            controller: 44,
            value: 127,
            channel: 0
        })
    );
}

#[test]
fn test_midi_raw() {
    assert_eq!(
        protocol::parse("midi:raw,176,44,127"),
        Some(Command::MidiRaw {
            bytes: [176, 44, 127]
        })
    );
}

#[test]
fn test_midi_note_on_default_channel() {
    assert_eq!(
        protocol::parse("midi:note_on,60,100"),
        Some(Command::MidiNoteOn {
            note: 60,
            velocity: 100,
            channel: 0
        })
    );
}

// --- WebSocket ---

#[test]
fn test_ws_broadcast() {
    assert_eq!(
        protocol::parse("ws:temperature,23.5"),
        Some(Command::WsBroadcast {
            id: "temperature".into(),
            value: "23.5".into()
        })
    );
}

#[test]
fn test_ws_raw() {
    assert_eq!(
        protocol::parse("ws:raw,{\"custom\":\"json\"}"),
        Some(Command::WsRaw("{\"custom\":\"json\"}".into()))
    );
}

// --- OSC ---

#[test]
fn test_osc_message() {
    assert_eq!(
        protocol::parse("osc:/sensor/temp,23.5"),
        Some(Command::OscMessage {
            address: "/sensor/temp".into(),
            args: vec!["23.5".into()]
        })
    );
}

#[test]
fn test_osc_no_args() {
    assert_eq!(
        protocol::parse("osc:/trigger"),
        Some(Command::OscMessage {
            address: "/trigger".into(),
            args: vec![]
        })
    );
}

#[test]
fn test_osc_multiple_args() {
    assert_eq!(
        protocol::parse("osc:/color,255,128,0"),
        Some(Command::OscMessage {
            address: "/color".into(),
            args: vec!["255".into(), "128".into(), "0".into()]
        })
    );
}

// --- Edge cases ---

#[test]
fn test_empty_line() {
    assert_eq!(protocol::parse(""), None);
}

#[test]
fn test_whitespace_only() {
    assert_eq!(protocol::parse("   "), None);
}

#[test]
fn test_unknown_prefix() {
    assert_eq!(protocol::parse("foo:bar,baz"), None);
}

#[test]
fn test_no_colon() {
    assert_eq!(protocol::parse("hello"), None);
}

#[test]
fn test_trimming() {
    assert_eq!(
        protocol::parse("  key:tap,a  \n"),
        Some(Command::KeyTap("a".into()))
    );
}
