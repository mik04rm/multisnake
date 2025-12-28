use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{error::Error, io};

pub fn run_room_selector() -> Result<Option<u32>, Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<Option<u32>, Box<dyn Error>> {
    let rooms = vec!["Room 1", "Room 2", "Room 3"];
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let title = Paragraph::new("MultiSnake Launcher")
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            let items: Vec<ListItem> = rooms
                .iter()
                .map(|r| ListItem::new(*r).style(Style::default().fg(Color::White)))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select Room"))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut list_state);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Up => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    rooms.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    KeyCode::Down => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i >= rooms.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
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
