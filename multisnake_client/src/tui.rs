use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use multisnake_shared::LobbyUpdate;
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{error::Error, io, thread};

pub fn run_room_selector() -> Result<Option<u32>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

use std::sync::mpsc;
use tungstenite::connect;

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<Option<u32>, Box<dyn Error>> {
    let mut rooms_count = vec![0; 3]; // TODO: minor check
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let (tx, rx) = mpsc::channel::<LobbyUpdate>();

    thread::spawn(move || {
        let url = "ws://127.0.0.1:8080/room";
        if let Ok((mut socket, _)) = connect(url) {
            loop {
                if let Ok(msg) = socket.read() {
                    if let tungstenite::Message::Text(text) = msg {
                        if let Ok(update) = serde_json::from_str::<LobbyUpdate>(&text) {
                            let _ = tx.send(update);
                        }
                    }
                } else {
                    break;
                }
            }
        }
    });

    loop {
        // Check for updates from network
        while let Ok(update) = rx.try_recv() {
            let idx = (update.room_id - 1) as usize;
            if idx < rooms_count.len() {
                rooms_count[idx] = update.player_count;
            }
        }

        // Drawing
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.area());

            let title = Paragraph::new("multisnake")
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Map room names to include player counts
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

        // Input handling
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Up => {
                        let i = list_state
                            .selected()
                            .map_or(0, |i| if i == 0 { 2 } else { i - 1 });
                        list_state.select(Some(i));
                    }
                    KeyCode::Down => {
                        let i = list_state
                            .selected()
                            .map_or(0, |i| if i >= 2 { 0 } else { i + 1 });
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
        }
    }
}
