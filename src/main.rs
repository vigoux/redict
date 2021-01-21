mod app;

#[macro_use]
extern crate clap;

use std::io;
use termion::raw::IntoRawMode;
use tui::{Terminal, Frame};
use tui::backend::{TermionBackend, Backend};
use tui::widgets::{Block, Borders, Paragraph, Tabs};
use tui::text::{Span, Spans};
use tui::style::{Style, Color};
use tui::layout::{Layout, Constraint, Direction, Rect};
use termion::event::Key;
use termion::input::TermRead;
use termion::screen::AlternateScreen;
use std::net::TcpStream;
use app::{App, HistoryMovement, AppMode};
use dictproto::url::DICTUrl;

fn make_block(name: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .title(name)
}

fn draw_definitions<B: Backend>(f: &mut Frame<B>, rect: Rect, app: &App) {

    let chunks = Layout::default()
        .constraints(
            [
            Constraint::Min(0),
            Constraint::Length(3),
            ].as_ref()
        ).split(rect);

    let titles = app.results.iter()
        .map(|res| {
            Spans::from(vec![Span::from(&*res.source.desc)])
        }).collect();

    let tabs = Tabs::new(titles)
        .block(make_block("Sources"))
        .highlight_style(Style::default().fg(Color::Blue))
        .select(app.selected_def());
    f.render_widget(tabs, chunks[1]);

    let text: Vec<Spans> = app.results.get(app.selected_def()).unwrap()
        .text.iter()
        .map(|line| { Spans::from(Span::from(String::from(line))) })
        .collect();

    let block = Paragraph::new(text)
        .block(make_block("Definition"))
        .scroll((app.definition_scroll(), 0));
    f.render_widget(block, chunks[0]);
}

fn draw_matches<B: Backend>(f: &mut Frame<B>, rect: Rect, app: &App) {
    let dbs: Vec<Spans> = app.matches.iter()
        .map(|m| {
            Spans::from(vec![Span::from(format!("{} ({})", m.word, m.source.name))])
        }).collect();

    let block = Paragraph::new(dbs)
        .block(Block::default().borders(Borders::ALL))
        .scroll((app.definition_scroll(), 0));
    f.render_widget(block, rect);
}


fn draw_databases<B: Backend>(f: &mut Frame<B>, rect: Rect, app: &App) {
    let dbs: Vec<Spans> = app.databases.iter()
        .map(|db| {
            Spans::from(vec![Span::from(format!("{} ({})", db.desc, db.name))])
        }).collect();

    let block = Paragraph::new(dbs)
        .block(Block::default().borders(Borders::ALL))
        .scroll((app.definition_scroll(), 0));
    f.render_widget(block, rect);
}

fn draw_strategies<B: Backend>(f: &mut Frame<B>, rect: Rect, app: &App) {
    let dbs: Vec<Spans> = app.stategies.iter()
        .map(|db| {
            Spans::from(vec![Span::from(format!("{} ({})", db.desc, db.name))])
        }).collect();

    let block = Paragraph::new(dbs)
        .block(Block::default().borders(Borders::ALL))
        .scroll((app.definition_scroll(), 0));
    f.render_widget(block, rect);
}

fn handle_key(key: &Key, app: &mut App) {
    match (app.mode(), key) {
        (AppMode::Define, Key::Right) => {
            app.next_definition();
        },
        (AppMode::Define, Key::Left) => {
            app.previous_definition();
        },

        // Enter mappings
        (AppMode::Define, Key::Char('\n')) => {
            app.run_define();
        },
        (AppMode::Databases, Key::Char('\n')) => {
            app.run_show_dbs();
        },
        (AppMode::Strategies, Key::Char('\n')) => {
            app.run_show_strats();
        },
        (AppMode::Match, Key::Char('\n')) => {
            app.run_match();
        }
        _ => {}
    }
}

fn main() -> Result<(), io::Error> {

    let validate_url = |url: String| -> Result<(), String> {
        DICTUrl::new(&url).map_err(|e| e.to_string())?;
        Ok(())
    };

    let matches = clap_app!(redict =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Connect and navigate DICT servers")
        (@arg SERVER: +required {validate_url} "Url to connect to")
    ).get_matches();

    let url = matches.value_of("SERVER").unwrap();

    let mut stdin = std::io::stdin().keys();

    // First answer
    let mut app = App::new(url);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {

            // Panes
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ].as_ref()
                )
                .split(f.size());


            // Search bar
            let block = Paragraph::new(app.searched.to_owned())
                .block(make_block("Search"));
            f.render_widget(block, chunks[0]);

            // Status section
            let block = Paragraph::new(
                if let Some(ref reply) = app.last_status {
                    reply.to_string()
                } else {
                    String::from("No status")
                }
                )
                .block(make_block("Status"));
            f.render_widget(block, chunks[1]);

            // Mode display
            let titles = AppMode::values().iter()
                .map(|mode| {
                    Spans::from(vec![Span::from(mode.text())])
                }).collect();
            let modes = Tabs::new(titles)
                .highlight_style(Style::default().fg(Color::Blue))
                .select(app.mode().into());
            f.render_widget(modes, chunks[2]);

            match app.mode() {
                AppMode::Define => {
                    draw_definitions(f, chunks[3], &app);
                },
                AppMode::Databases => {
                    draw_databases(f, chunks[3], &app);
                },
                AppMode::Strategies => {
                    draw_strategies(f, chunks[3], &app);
                }
                AppMode::Match => {
                    draw_matches(f, chunks[3], &app);
                }
            }

        })?;

        if let Some(Ok(evt)) = stdin.next() {
            match evt {
                Key::Esc => { break ;},

                // Scrolling
                Key::PageUp | Key::Up => {
                    app.scroll_up();
                },
                Key::PageDown | Key::Down => {
                    app.scroll_down();
                },

                // Mode management
                Key::Char('\t') => {
                    app.set_mode(app.mode().next());
                },
                Key::BackTab => {
                    app.set_mode(app.mode().previous());
                }

                // Search management
                Key::Char(c) if c != '\n' => {
                    app.searched.push(c);
                },
                Key::Backspace => {
                    app.searched.pop();
                },
                Key::Ctrl('u') => {
                    app.searched = String::new();
                },

                // History management
                Key::Ctrl('h') => {
                    app.history_goto(HistoryMovement::Previous);
                },
                Key::Ctrl('l') => {
                    app.history_goto(HistoryMovement::Next);
                }

                // Other keys depend on the mode
                key => handle_key(&key, &mut app)
            }
        } else {
            break;
        }
    }

    Ok(())
}

