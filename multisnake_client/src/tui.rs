use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures_util::StreamExt;
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{error::Error, io};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use multisnake_shared::{LobbyUpdate, N_ROOMS};

pub async fn run_room_selector(server_addr: &str) -> Result<Option<u32>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, server_addr).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    server_addr: &str,
) -> Result<Option<u32>, Box<dyn Error>> {
    let mut rooms_count = [0; N_ROOMS as usize]; // TODO
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let mut event_stream = EventStream::new();

    let url = format!("ws://{}/room", server_addr);

    let (ws_stream, _) = connect_async(url).await?;
    let (_, mut ws_rx) = ws_stream.split();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(f.area());

            let title = Paragraph::new("multisnake")
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            let items: Vec<ListItem> = rooms_count
                .iter()
                .enumerate()
                .map(|(i, count)| {
                    let content = format!("Room {}  [{} players]", i + 1, count);
                    ListItem::new(content).style(Style::default().fg(Color::White))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select room"))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut list_state);
        })?;

        tokio::select! {
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                            KeyCode::Up => {
                                let i = list_state.selected().map_or(0, |i| if i == 0 { 2 } else { i - 1 });
                                list_state.select(Some(i));
                            }
                            KeyCode::Down => {
                                let i = list_state.selected().map_or(0, |i| if i >= 2 { 0 } else { i + 1 });
                                list_state.select(Some(i));
                            }
                            KeyCode::Enter => {
                                if let Some(i) = list_state.selected() {
                                    return Ok(Some((i + 1) as u32));
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            maybe_message = ws_rx.next() => {
                match maybe_message {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(update) = serde_json::from_str::<LobbyUpdate>(&text) {
                            let idx = (update.room_id - 1) as usize;
                            if idx < rooms_count.len() {
                                rooms_count[idx] = update.player_count;
                            }
                        }
                    }
                    None => {
                        // TODO: dont stop running TUI on disconnect
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(None)
}
