use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read},
    time::Duration,
};

struct App {
    items: Vec<Word>,
    state: ListState,
}
impl App {
    fn new() -> Self {
        let mut raw = String::new();
        fs::File::open("./word.json")
            .unwrap()
            .read_to_string(&mut raw)
            .unwrap();
        let items: Vec<Word> = serde_json::from_str(&raw).unwrap();
        let state = ListState::default();
        Self { items, state }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct Word {
    word: String,
    oldword: String,
    strokes: String,
    pinyin: String,
    radicals: String,
    explanation: String,
    more: String,
}
fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        if event::poll(Duration::from_millis(1000))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => app.previous(),
                    KeyCode::Down => app.next(),
                    KeyCode::Char('q') => break,
                    _ => (),
                }
            }
        };
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(6), Constraint::Ratio(1, 1)].as_ref())
        .split(f.size());
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|word| ListItem::new(Line::from(word.word.as_str())))
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("目录"))
        .highlight_style(
            Style::default()
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    let select = &app.items[app.state.selected().unwrap_or_default()];
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(Vec::from([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Ratio(1, 1),
        ]))
        .split(main_chunks[1]);
    let layout_min = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Vec::from([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
        ]))
        .split(layout[0]);
    f.render_widget(
        Paragraph::new(select.word.as_str())
            .block(Block::default().title("简体").borders(Borders::ALL)),
        layout_min[0],
    );
    f.render_widget(
        Paragraph::new(select.oldword.as_str())
            .block(Block::default().title("繁体").borders(Borders::ALL)),
        layout_min[1],
    );
    f.render_widget(
        Paragraph::new(select.strokes.as_str())
            .block(Block::default().title("笔画").borders(Borders::ALL)),
        layout_min[2],
    );
    f.render_widget(
        Paragraph::new(select.pinyin.as_str())
            .block(Block::default().title("拼音").borders(Borders::ALL)),
        layout_min[3],
    );
    f.render_widget(
        Paragraph::new(select.radicals.as_str())
            .block(Block::default().title("字基").borders(Borders::ALL)),
        layout_min[4],
    );
    f.render_widget(
        Paragraph::new(select.explanation.as_str())
            .block(Block::default().title("解释").borders(Borders::ALL)),
        layout[1],
    );
    f.render_widget(
        Paragraph::new(select.more.as_str())
            .block(Block::default().title("更多").borders(Borders::ALL)),
        layout[2],
    );
    f.render_stateful_widget(list, main_chunks[0], &mut app.state);
}
