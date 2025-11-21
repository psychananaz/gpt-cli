use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use tokio::runtime::Runtime;

use crate::gpt::GPTClient;

#[derive(Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub status: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            status: Some("Press Esc or Ctrl+C to exit".to_string()),
        }
    }

    fn push_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: "You".to_string(),
            content,
        });
    }

    fn push_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: "GPT".to_string(),
            content,
        });
    }
}

pub fn run(runtime: &Runtime, client: &mut GPTClient, app: &mut App) -> crossterm::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_event_loop(runtime, client, app, &mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop(
    runtime: &Runtime,
    client: &mut GPTClient,
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> crossterm::Result<()> {
    loop {
        terminal.draw(|frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Min(5),
                        Constraint::Length(1),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let items: Vec<ListItem> = app
                .messages
                .iter()
                .map(|m| {
                    let style = match m.role.as_str() {
                        "You" => Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                        _ => Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    };
                    let text = Text::from(vec![Line::from(vec![
                        Span::styled(format!("{}: ", m.role), style),
                        Span::raw(&m.content),
                    ])]);
                    ListItem::new(text)
                })
                .collect();

            let conversation = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Conversation"));
            frame.render_widget(conversation, chunks[0]);

            let status_text = app.status.as_ref().map(|s| s.as_str()).unwrap_or("Ready");
            let status_widget = Paragraph::new(status_text)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Status"));
            frame.render_widget(status_widget, chunks[1]);

            let input = Paragraph::new(app.input.as_str())
                .style(Style::default())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Enter a prompt and press Enter"),
                );
            frame.render_widget(input, chunks[2]);
            frame.set_cursor(chunks[2].x + app.input.len() as u16 + 1, chunks[2].y + 1);
        })?;

        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        break;
                    }
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Enter => {
                            let prompt = app.input.trim().to_string();
                            if prompt.is_empty() {
                                continue;
                            }

                            app.push_user_message(prompt.clone());
                            app.input.clear();
                            app.status = Some("Thinking...".to_string());

                            let response = runtime.block_on(client.get_response(&prompt));
                            match response {
                                Ok(message) => {
                                    app.push_assistant_message(message);
                                    app.status = Some("Ready".to_string());
                                }
                                Err(error) => {
                                    app.status = Some(format!("Error: {}", error));
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
