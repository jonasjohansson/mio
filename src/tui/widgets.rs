//! Custom TUI widgets: selection popups and help overlay.

use crate::bridge::midi::MidiPortInfo;
use crate::serial::PortInfo;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Active popup state.
#[derive(Debug, Clone)]
pub enum Popup {
    PortSelect {
        ports: Vec<PortInfo>,
        selected: usize,
    },
    MidiSelect {
        ports: Vec<MidiPortInfo>,
        selected: usize,
    },
    Help,
}

/// Render a centered selection popup.
pub fn render_selection_popup(frame: &mut Frame, title: &str, items: &[String], selected: usize) {
    let area = centered_rect(50, 60, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!("  {}  ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == selected { "▸ " } else { "  " };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, item),
                style,
            )))
        })
        .collect();

    let list = List::new(list_items).block(block);
    frame.render_widget(list, area);
}

/// Render the help popup.
pub fn render_help_popup(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("  Help  ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Mio — Serial-to-Everything Bridge",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [q]     ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled("  [c]     ", Style::default().fg(Color::Yellow)),
            Span::raw("Connect/disconnect serial port"),
        ]),
        Line::from(vec![
            Span::styled("  [m]     ", Style::default().fg(Color::Yellow)),
            Span::raw("Connect/disconnect MIDI output"),
        ]),
        Line::from(vec![
            Span::styled("  [↑/↓]   ", Style::default().fg(Color::Yellow)),
            Span::raw("Scroll log"),
        ]),
        Line::from(vec![
            Span::styled("  [Esc]   ", Style::default().fg(Color::Yellow)),
            Span::raw("Close popup"),
        ]),
        Line::from(vec![
            Span::styled("  [Ctrl+C]", Style::default().fg(Color::Yellow)),
            Span::raw(" Force quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Protocol: PREFIX:SUBCOMMAND,arg1,arg2,...",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  key:tap,a          Press and release 'a'"),
        Line::from("  key:down,space     Hold space"),
        Line::from("  key:up,space       Release space"),
        Line::from("  key:type,hello     Type 'hello'"),
        Line::from("  mouse:move,100,200 Move mouse to (100,200)"),
        Line::from("  mouse:click,left   Left click"),
        Line::from("  midi:note_on,60,127,0  MIDI note on"),
        Line::from("  ws:temp,23.5       WebSocket broadcast"),
        Line::from("  osc:/addr,1.0      Send OSC message"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press [Esc] to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    frame.render_widget(Paragraph::new(help_text).block(block), area);
}

/// Create a centered rectangle with a given percentage of the parent area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
