use std::path::PathBuf;
use std::time::{Duration, Instant};

use ansi_to_tui::IntoText;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use notify::{RecursiveMode, Watcher};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;

use crate::cli::{ColorMode, ThemeName, WatchArgs};
use crate::parse::parse_markdown;
use crate::render::{self, RenderContext};
use crate::style::theme::Theme;
use crate::terminal::TerminalInfo;

struct App {
    content: Text<'static>,
    scroll: u16,
    content_height: u16,
    filename: String,
}

fn render_markdown(path: &PathBuf, term_width: u16) -> Text<'static> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Text::raw(format!("Error reading file: {}", e)),
    };

    let color_mode = ColorMode::Auto;
    let term = TerminalInfo::detect(&color_mode, Some(term_width));
    let theme = Theme::from_name(&ThemeName::Dark);
    let arena = typed_arena::Arena::new();
    let root = parse_markdown(&arena, &content);

    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = RenderContext::new(&term, &theme, "base16-ocean.dark".to_string(), false);
    if render::render(&mut buf, root, &mut ctx).is_err() {
        return Text::raw("Error rendering markdown");
    }

    match buf.into_text() {
        Ok(text) => text,
        Err(_) => Text::raw("Error converting ANSI output"),
    }
}

pub fn watch(args: &WatchArgs) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(&args.file)
        .canonicalize()
        .map_err(|e| format!("Cannot open '{}': {}", args.file, e))?;

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| args.file.clone());

    let mut terminal = ratatui::init();
    let size = terminal.size()?;

    // Leave 1 row for status bar
    let content_width = size.width.saturating_sub(0);

    let content = render_markdown(&file_path, content_width);
    let mut app = App {
        content_height: content.lines.len() as u16,
        content,
        scroll: 0,
        filename,
    };

    // Set up file watcher via mpsc
    let (file_tx, file_rx) = std::sync::mpsc::channel();
    let watch_path = file_path.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        if let Ok(ev) = res
            && ev.kind.is_modify()
        {
            let _ = file_tx.send(());
        }
    })?;
    watcher.watch(&watch_path, RecursiveMode::NonRecursive)?;

    let mut last_change = Instant::now();

    loop {
        terminal.draw(|frame| {
            let chunks =
                Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(frame.area());

            let content_area = chunks[0];
            let status_area = chunks[1];

            // Clamp scroll
            let max_scroll = app.content_height.saturating_sub(content_area.height);
            if app.scroll > max_scroll {
                app.scroll = max_scroll;
            }

            let paragraph = Paragraph::new(app.content.clone()).scroll((app.scroll, 0));
            frame.render_widget(paragraph, content_area);

            // Status bar
            let current_line = app.scroll + 1;
            let total = app.content_height;
            let status_text = format!(
                " {} | {}/{} | q quit | j/k scroll | d/u page",
                app.filename, current_line, total
            );
            let status = Paragraph::new(status_text).style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(status, status_area);
        })?;

        // Poll for keyboard input (250ms timeout)
        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => {
                    app.scroll = app.scroll.saturating_add(1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.scroll = app.scroll.saturating_sub(1);
                }
                KeyCode::Char('d') | KeyCode::PageDown | KeyCode::Char(' ') => {
                    let half = terminal.size()?.height.saturating_sub(1) / 2;
                    app.scroll = app.scroll.saturating_add(half);
                }
                KeyCode::Char('u') | KeyCode::PageUp => {
                    let half = terminal.size()?.height.saturating_sub(1) / 2;
                    app.scroll = app.scroll.saturating_sub(half);
                }
                KeyCode::Char('g') | KeyCode::Home => {
                    app.scroll = 0;
                }
                KeyCode::Char('G') | KeyCode::End => {
                    app.scroll = u16::MAX; // will be clamped in draw
                }
                _ => {}
            }
        }

        // Check for file changes
        if file_rx.try_recv().is_ok() {
            // Drain any extra queued notifications
            while file_rx.try_recv().is_ok() {}

            if last_change.elapsed() >= Duration::from_millis(300) {
                last_change = Instant::now();
                let width = terminal.size()?.width;
                app.content = render_markdown(&file_path, width);
                app.content_height = app.content.lines.len() as u16;
            }
        }
    }

    ratatui::restore();
    Ok(())
}
