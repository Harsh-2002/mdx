use std::path::PathBuf;
use std::time::Duration;

use ansi_to_tui::IntoText;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;

use crate::cli::{ColorMode, PresentArgs, ThemeName};
use crate::parse::parse_markdown;
use crate::render::{self, RenderContext};
use crate::style::theme::Theme;
use crate::terminal::TerminalInfo;

struct App {
    slides: Vec<Text<'static>>,
    current: usize,
    scroll: u16,
}

fn render_slide(markdown: &str, term_width: u16) -> Text<'static> {
    let color_mode = ColorMode::Auto;
    let term = TerminalInfo::detect(&color_mode, Some(term_width));
    let theme = Theme::from_name(&ThemeName::Dark);
    let arena = typed_arena::Arena::new();
    let root = parse_markdown(&arena, markdown);

    let mut buf: Vec<u8> = Vec::new();
    let mut ctx = RenderContext::new(&term, &theme, "base16-ocean.dark".to_string(), false);
    if render::render(&mut buf, root, &mut ctx).is_err() {
        return Text::raw("Error rendering slide");
    }

    match buf.into_text() {
        Ok(text) => text,
        Err(_) => Text::raw("Error converting slide output"),
    }
}

pub fn present(args: &PresentArgs) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(&args.file)
        .canonicalize()
        .map_err(|e| format!("Cannot open '{}': {}", args.file, e))?;

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Cannot read '{}': {}", args.file, e))?;

    // Split on \n---\n to create slides
    let slide_texts: Vec<&str> = content.split("\n---\n").collect();

    if slide_texts.is_empty() {
        return Err("No slides found".into());
    }

    let mut terminal = ratatui::init();
    let size = terminal.size()?;

    let slides: Vec<Text<'static>> = slide_texts
        .iter()
        .map(|s| render_slide(s.trim(), size.width))
        .collect();

    let mut app = App {
        slides,
        current: 0,
        scroll: 0,
    };

    loop {
        terminal.draw(|frame| {
            let chunks =
                Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(frame.area());

            let content_area = chunks[0];
            let status_area = chunks[1];

            if let Some(slide) = app.slides.get(app.current) {
                let slide_height = slide.lines.len() as u16;

                // Center vertically if slide is shorter than the area
                let vertical_offset = if slide_height < content_area.height {
                    (content_area.height - slide_height) / 2
                } else {
                    0
                };

                let max_scroll = slide_height.saturating_sub(content_area.height);
                if app.scroll > max_scroll {
                    app.scroll = max_scroll;
                }

                let paragraph = Paragraph::new(slide.clone())
                    .scroll((app.scroll.saturating_sub(vertical_offset), 0));

                if vertical_offset > 0 && app.scroll == 0 {
                    // Center: add padding at top
                    let padded_chunks =
                        Layout::vertical([Constraint::Length(vertical_offset), Constraint::Min(1)])
                            .split(content_area);
                    let paragraph = Paragraph::new(slide.clone());
                    frame.render_widget(paragraph, padded_chunks[1]);
                } else {
                    frame.render_widget(paragraph, content_area);
                }
            }

            // Status bar
            let status_text = format!(
                " Slide {}/{} | \u{2190} \u{2192} navigate | q quit",
                app.current + 1,
                app.slides.len()
            );
            let status = Paragraph::new(status_text)
                .style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Left);
            frame.render_widget(status, status_area);
        })?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') | KeyCode::Enter => {
                    if app.current + 1 < app.slides.len() {
                        app.current += 1;
                        app.scroll = 0;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if app.current > 0 {
                        app.current -= 1;
                        app.scroll = 0;
                    }
                }
                KeyCode::Char('g') | KeyCode::Home => {
                    app.current = 0;
                    app.scroll = 0;
                }
                KeyCode::Char('G') | KeyCode::End => {
                    app.current = app.slides.len().saturating_sub(1);
                    app.scroll = 0;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.scroll = app.scroll.saturating_add(1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.scroll = app.scroll.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    ratatui::restore();
    Ok(())
}
