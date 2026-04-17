use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{error::Error, io, time::Duration};
use tokio::sync::mpsc;

mod arduino_cli;
use arduino_cli::LibraryInfo;

#[derive(PartialEq)]
enum AppMode {
    Normal,
    Search,
    Help,
}

enum AppEvent {
    Input(CEvent),
    Tick,
    LibrariesLoaded(Vec<LibraryInfo>),
    LibraryInstalled(String),
    LibraryUninstalled(String),
    CommandError(String),
}

struct App {
    mode: AppMode,
    search_input: String,
    libraries: Vec<LibraryInfo>,
    list_state: ListState,
    status_message: String,
    is_loading: bool,
}

impl App {
    fn new() -> App {
        App {
            mode: AppMode::Normal,
            search_input: String::new(),
            libraries: Vec::new(),
            list_state: ListState::default(),
            status_message:
                "Press '?' for help, '/' to search, 'i' to install, 'u' to uninstall, 'q' to quit"
                    .to_string(),
            is_loading: true,
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.libraries.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.libraries.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let (tx, mut rx) = mpsc::channel(100);

    // Initial load
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        match arduino_cli::list_installed_libraries().await {
            Ok(libs) => {
                let _ = tx_clone.send(AppEvent::LibrariesLoaded(libs)).await;
            }
            Err(e) => {
                let _ = tx_clone.send(AppEvent::CommandError(e)).await;
            }
        }
    });

    // Event loop
    let tick_rate = Duration::from_millis(200);
    let tx_input = tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(tick_rate).unwrap() {
                if let Ok(event) = event::read() {
                    let _ = tx_input.send(AppEvent::Input(event)).await;
                }
            }
            let _ = tx_input.send(AppEvent::Tick).await;
        }
    });

    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Input(CEvent::Key(key)) => {
                    if key.kind == KeyEventKind::Press {
                        match app.mode {
                            AppMode::Normal => match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Char('j') | KeyCode::Down => app.next(),
                                KeyCode::Char('k') | KeyCode::Up => app.previous(),
                                KeyCode::Char('?') | KeyCode::Char('h') => {
                                    app.mode = AppMode::Help;
                                }
                                KeyCode::Char('/') => {
                                    app.mode = AppMode::Search;
                                }
                                KeyCode::Esc => {
                                    if !app.search_input.is_empty() {
                                        app.search_input.clear();
                                        app.status_message = "Loading installed libraries...".to_string();
                                        app.is_loading = true;
                                        let tx_load = tx.clone();
                                        tokio::spawn(async move {
                                            match arduino_cli::list_installed_libraries().await {
                                                Ok(libs) => {
                                                    let _ = tx_load.send(AppEvent::LibrariesLoaded(libs)).await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_load.send(AppEvent::CommandError(e)).await;
                                                }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Char('i') => {
                                    if let Some(selected) = app.list_state.selected() {
                                        if let Some(lib) = app.libraries.get(selected) {
                                            let lib_name = lib.name.clone();
                                            let tx_install = tx.clone();
                                            app.status_message =
                                                format!("Installing {}...", lib_name);
                                            app.is_loading = true;
                                            tokio::spawn(async move {
                                                match arduino_cli::install_library(&lib_name).await
                                                {
                                                    Ok(_) => {
                                                        let _ = tx_install
                                                            .send(AppEvent::LibraryInstalled(
                                                                lib_name,
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx_install
                                                            .send(AppEvent::CommandError(e))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                KeyCode::Char('u') => {
                                    if let Some(selected) = app.list_state.selected() {
                                        if let Some(lib) = app.libraries.get(selected) {
                                            let lib_name = lib.name.clone();
                                            let tx_uninstall = tx.clone();
                                            app.status_message =
                                                format!("Uninstalling {}...", lib_name);
                                            app.is_loading = true;
                                            tokio::spawn(async move {
                                                match arduino_cli::uninstall_library(&lib_name)
                                                    .await
                                                {
                                                    Ok(_) => {
                                                        let _ = tx_uninstall
                                                            .send(AppEvent::LibraryUninstalled(
                                                                lib_name,
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx_uninstall
                                                            .send(AppEvent::CommandError(e))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                _ => {}
                            },
                            AppMode::Search => match key.code {
                                KeyCode::Enter => {
                                    app.mode = AppMode::Normal;
                                    let query = app.search_input.clone();
                                    if query.is_empty() {
                                        app.status_message = "Loading installed libraries...".to_string();
                                    } else {
                                        app.status_message = format!("Searching for '{}'...", query);
                                    }
                                    app.is_loading = true;
                                    let tx_search = tx.clone();
                                    tokio::spawn(async move {
                                        let result = if query.is_empty() {
                                            arduino_cli::list_installed_libraries().await
                                        } else {
                                            arduino_cli::search_libraries(&query).await
                                        };

                                        match result {
                                            Ok(libs) => {
                                                let _ = tx_search
                                                    .send(AppEvent::LibrariesLoaded(libs))
                                                    .await;
                                            }
                                            Err(e) => {
                                                let _ =
                                                    tx_search.send(AppEvent::CommandError(e)).await;
                                            }
                                        }
                                    });
                                }
                                KeyCode::Char(c) => {
                                    app.search_input.push(c);
                                }
                                KeyCode::Backspace => {
                                    app.search_input.pop();
                                }
                                KeyCode::Esc => {
                                    app.mode = AppMode::Normal;
                                    if !app.search_input.is_empty() {
                                        app.search_input.clear();
                                        app.status_message = "Loading installed libraries...".to_string();
                                        app.is_loading = true;
                                        let tx_load = tx.clone();
                                        tokio::spawn(async move {
                                            match arduino_cli::list_installed_libraries().await {
                                                Ok(libs) => {
                                                    let _ = tx_load.send(AppEvent::LibrariesLoaded(libs)).await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_load.send(AppEvent::CommandError(e)).await;
                                                }
                                            }
                                        });
                                    }
                                }
                                _ => {}
                            },
                            AppMode::Help => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('h') => {
                                    app.mode = AppMode::Normal;
                                }
                                _ => {}
                            },
                        }
                    }
                }
                AppEvent::Input(_) => {}
                AppEvent::LibrariesLoaded(libs) => {
                    // Update installed status if we did a search
                    if !app.search_input.is_empty() {
                        // A simplistic way to check if installed: fire another request to get installed
                        // We will leave them as `is_installed: false` for search results for now to keep it fast
                    }
                    app.libraries = libs;
                    app.list_state.select(if app.libraries.is_empty() {
                        None
                    } else {
                        Some(0)
                    });
                    app.is_loading = false;
                    app.status_message = format!("Found {} libraries.", app.libraries.len());
                }
                AppEvent::LibraryInstalled(name) => {
                    app.is_loading = false;
                    app.status_message = format!("Successfully installed {}.", name);
                    if let Some(lib) = app.libraries.iter_mut().find(|l| l.name == name) {
                        lib.is_installed = true;
                    }
                }
                AppEvent::LibraryUninstalled(name) => {
                    app.is_loading = false;
                    app.status_message = format!("Successfully uninstalled {}.", name);
                    if let Some(lib) = app.libraries.iter_mut().find(|l| l.name == name) {
                        lib.is_installed = false;
                    }
                }
                AppEvent::CommandError(e) => {
                    app.is_loading = false;
                    app.status_message = format!("Error: {}", e);
                }
                AppEvent::Tick => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    // Search Box
    let search_style = match app.mode {
        AppMode::Search => Style::default().fg(Color::Yellow),
        AppMode::Normal | AppMode::Help => Style::default(),
    };
    let search_title = match app.mode {
        AppMode::Search => " Search (Press Enter to search, Esc to cancel) ",
        AppMode::Normal => " Search (Press '/' to search) ",
        AppMode::Help => "(Press 'esc' to go back) ",
    };
    let search_text = Paragraph::new(app.search_input.as_str())
        .style(search_style)
        .block(Block::default().borders(Borders::ALL).title(search_title));
    f.render_widget(search_text, chunks[0]);

    // Main area layout (List + Details)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[1]);

    // Library List
    let items: Vec<ListItem> = app
        .libraries
        .iter()
        .map(|l| {
            let status = if l.is_installed { "[I] " } else { "[ ] " };
            let line = Line::from(vec![
                Span::styled(
                    status,
                    Style::default().fg(if l.is_installed {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::raw(l.name.clone()),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Libraries "))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

    // Details Pane
    let details_text = if let Some(selected) = app.list_state.selected() {
        if let Some(lib) = app.libraries.get(selected) {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&lib.name),
                ]),
                Line::from(vec![
                    Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&lib.version),
                ]),
            ];

            if let Some(author) = &lib.author {
                text.push(Line::from(vec![
                    Span::styled("Author: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(author),
                ]));
            }
            if let Some(cat) = &lib.category {
                text.push(Line::from(vec![
                    Span::styled("Category: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(cat),
                ]));
            }
            if let Some(sentence) = &lib.sentence {
                text.push(Line::from(""));
                text.push(Line::from(Span::styled(
                    "Description:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                text.push(Line::from(Span::raw(sentence)));
            }

            text.push(Line::from(""));
            if lib.is_installed {
                text.push(Line::from(Span::styled(
                    "Status: Installed",
                    Style::default().fg(Color::Green),
                )));
            } else {
                text.push(Line::from(Span::styled(
                    "Status: Not Installed",
                    Style::default().fg(Color::DarkGray),
                )));
            }

            text
        } else {
            vec![Line::from("No library selected")]
        }
    } else {
        vec![Line::from("No library selected")]
    };

    let details = Paragraph::new(details_text)
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .wrap(Wrap { trim: true });

    f.render_widget(details, main_chunks[1]);

    // Status Box
    let status_style = if app.is_loading {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::RAPID_BLINK)
    } else {
        Style::default().fg(Color::Gray)
    };

    let status = Paragraph::new(app.status_message.as_str())
        .style(status_style)
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(status, chunks[2]);

    // Help Popup
    if let AppMode::Help = app.mode {
        let block = Block::default()
            .title(" Help (Press Esc to close) ")
            .borders(Borders::ALL);
        let area = centered_rect(60, 60, f.area());
        f.render_widget(Clear, area); //this clears out the background

        let help_text = vec![
            Line::from(Span::styled(
                "Navigation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  j / Down   : Move down"),
            Line::from("  k / Up     : Move up"),
            Line::from(""),
            Line::from(Span::styled(
                "Actions",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  /          : Search for a library"),
            Line::from("  i          : Install selected library"),
            Line::from("  u          : Uninstall selected library"),
            Line::from(""),
            Line::from(Span::styled(
                "General",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  ? / h      : Toggle this help menu"),
            Line::from("  q          : Quit application"),
        ];

        let help_paragraph = Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: true });

        f.render_widget(help_paragraph, area);
    }
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
