//! TUI layout: status bar, log area, and footer.

use crate::app::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use super::widgets;

/// Render the full TUI layout.
pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Status area
            Constraint::Min(5),    // Log area
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    render_status(frame, chunks[0], state);
    render_log(frame, chunks[1], state);
    render_footer(frame, chunks[2], state);

    // Render popup overlay if active
    if let Some(popup) = &state.active_popup {
        match popup {
            widgets::Popup::PortSelect { ports, selected } => {
                let items: Vec<String> = ports.iter().map(|p| format!("{} ({})", p.name, p.port_type)).collect();
                widgets::render_selection_popup(frame, "Select Serial Port", &items, *selected);
            }
            widgets::Popup::MidiSelect { ports, selected } => {
                let items: Vec<String> = ports.iter().map(|p| p.name.clone()).collect();
                widgets::render_selection_popup(frame, "Select MIDI Port", &items, *selected);
            }
            widgets::Popup::Help => {
                widgets::render_help_popup(frame);
            }
        }
    }
}

fn render_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(format!("  Mio v{}  ", env!("CARGO_PKG_VERSION")))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let status_lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Serial status
    let serial_status = if state.serial_connected {
        let port = state.serial_port_name.as_deref().unwrap_or("?");
        Line::from(vec![
            Span::styled("  Serial  ", Style::default().fg(Color::White)),
            Span::styled(format!("[{}]  {}  ", port, state.baud_rate), Style::default().fg(Color::Cyan)),
            Span::styled("● CONNECTED", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  Serial  ", Style::default().fg(Color::White)),
            Span::styled("○ DISCONNECTED", Style::default().fg(Color::DarkGray)),
        ])
    };
    frame.render_widget(Paragraph::new(serial_status), status_lines[0]);

    // MIDI status
    let midi_status = if let Some(name) = &state.midi_port_name {
        Line::from(vec![
            Span::styled("  MIDI    ", Style::default().fg(Color::White)),
            Span::styled(format!("[{}]  ", name), Style::default().fg(Color::Magenta)),
            Span::styled("● CONNECTED", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  MIDI    ", Style::default().fg(Color::White)),
            Span::styled("○ DISCONNECTED", Style::default().fg(Color::DarkGray)),
        ])
    };
    frame.render_widget(Paragraph::new(midi_status), status_lines[1]);

    // WebSocket status
    let ws_status = if state.ws_enabled {
        let clients = state.ws_client_count;
        Line::from(vec![
            Span::styled("  WS      ", Style::default().fg(Color::White)),
            Span::styled(format!(":{} ({} clients)  ", state.ws_port, clients), Style::default().fg(Color::Yellow)),
            Span::styled("● LISTENING", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  WS      ", Style::default().fg(Color::White)),
            Span::styled("○ DISABLED", Style::default().fg(Color::DarkGray)),
        ])
    };
    frame.render_widget(Paragraph::new(ws_status), status_lines[2]);

    // OSC status
    let osc_status = if state.osc_enabled {
        Line::from(vec![
            Span::styled("  OSC     ", Style::default().fg(Color::White)),
            Span::styled(format!("{}  ", state.osc_remote), Style::default().fg(Color::Blue)),
            Span::styled("● READY", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  OSC     ", Style::default().fg(Color::White)),
            Span::styled("○ DISABLED", Style::default().fg(Color::DarkGray)),
        ])
    };
    frame.render_widget(Paragraph::new(osc_status), status_lines[3]);
}

fn render_log(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;
    let total_lines = state.log_lines.len();

    // Calculate which lines to show based on scroll position
    let start = if total_lines > visible_height {
        total_lines - visible_height - state.scroll_offset.min(total_lines.saturating_sub(visible_height))
    } else {
        0
    };
    let end = (start + visible_height).min(total_lines);

    let lines: Vec<Line> = state.log_lines[start..end]
        .iter()
        .map(|entry| {
            let mut spans = vec![];
            if state.show_timestamps {
                spans.push(Span::styled(
                    format!("  {}  ", entry.timestamp),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            spans.push(Span::styled(
                &entry.raw_line,
                Style::default().fg(Color::White),
            ));
            spans.push(Span::raw("  ->  "));
            spans.push(Span::styled(
                &entry.result,
                Style::default().fg(Color::Cyan),
            ));
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);

    // Scrollbar
    if total_lines > visible_height {
        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height))
            .position(total_lines.saturating_sub(visible_height).saturating_sub(state.scroll_offset));
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area,
            &mut scrollbar_state,
        );
    }
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let connect_label = if state.serial_connected { "Disconnect" } else { "Connect" };
    let midi_label = if state.midi_port_name.is_some() { "MIDI off" } else { "MIDI" };

    let footer = Line::from(vec![
        Span::styled("  [q]", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit  "),
        Span::styled("[c]", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" {}  ", connect_label)),
        Span::styled("[m]", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" {}  ", midi_label)),
        Span::styled("[↑↓]", Style::default().fg(Color::Yellow)),
        Span::raw(" Scroll  "),
        Span::styled("[?]", Style::default().fg(Color::Yellow)),
        Span::raw(" Help"),
    ]);

    frame.render_widget(Paragraph::new(footer).block(block), area);
}
