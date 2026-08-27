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

/// Split a document into slides on top-level `---` separators.
///
/// A naive `split("\n---\n")` runs before any parsing, which makes YAML front
/// matter leak in as slide 1, turns a `---` inside a fenced code block into a
/// slide break, and mistakes a setext H2 underline for one. This walks lines
/// instead, tracking fence state, and only breaks where CommonMark would parse
/// a thematic break rather than something else.
fn split_slides(content: &str) -> Vec<String> {
    let body = crate::frontmatter::strip(content);

    let mut slides = Vec::new();
    let mut current = String::new();
    let mut fence: Option<(char, usize)> = None;
    let mut prev_blank = true; // start of document counts as a blank line

    for line in body.lines() {
        let trimmed = line.trim_start();
        // Track fenced code blocks so their contents are never scanned for breaks.
        let fence_char = trimmed.chars().next().filter(|c| *c == '`' || *c == '~');
        if let Some(fc) = fence_char {
            let run = trimmed.chars().take_while(|c| *c == fc).count();
            if run >= 3 {
                match fence {
                    // A closing fence must match the opening char and be at least
                    // as long; anything else is content.
                    Some((open_c, open_len)) if open_c == fc && run >= open_len => fence = None,
                    None => fence = Some((fc, run)),
                    _ => {}
                }
            }
        }

        // A `---` directly under a non-blank line is a setext H2 underline, not a
        // thematic break, so it must not split.
        let is_break = fence.is_none() && prev_blank && is_slide_break(line);

        if is_break {
            slides.push(std::mem::take(&mut current));
        } else {
            current.push_str(line);
            current.push('\n');
        }
        prev_blank = line.trim().is_empty();
    }
    slides.push(current);

    slides
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// True for a `---` slide separator: three or more dashes, optionally separated
/// by spaces, indented less than four columns (four or more is an indented code
/// block).
///
/// Deliberately dash-only. CommonMark also treats `***` and `___` as thematic
/// breaks, but the previous `split("\n---\n")` never split on those, and decks
/// that use them as an in-slide rule must keep working.
fn is_slide_break(line: &str) -> bool {
    let indent = line.len() - line.trim_start().len();
    if indent >= 4 {
        return false;
    }
    let t = line.trim();
    t.chars().filter(|ch| !ch.is_whitespace()).count() >= 3
        && !t.is_empty()
        && t.chars().all(|ch| ch == '-' || ch.is_whitespace())
}

pub fn present(args: &PresentArgs) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(&args.file)
        .canonicalize()
        .map_err(|e| format!("Cannot open '{}': {}", args.file, e))?;

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Cannot read '{}': {}", args.file, e))?;

    let slide_texts = split_slides(&content);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_front_matter_does_not_become_a_slide() {
        let doc = "---\ntitle: Deck\nauthor: Me\n---\n\n# One\n\n---\n\n# Two\n";
        let slides = split_slides(doc);
        assert_eq!(slides.len(), 2, "got: {:?}", slides);
        assert!(slides[0].starts_with("# One"), "got: {:?}", slides[0]);
        assert!(!slides.iter().any(|s| s.contains("title: Deck")));
    }

    #[test]
    fn test_dashes_inside_code_fence_do_not_split() {
        let doc = "# One\n\n```yaml\na: 1\n---\nb: 2\n```\n\n---\n\n# Two\n";
        let slides = split_slides(doc);
        assert_eq!(slides.len(), 2, "got: {:?}", slides);
        assert!(slides[0].contains("---"), "fence content must survive");
        assert!(slides[1].starts_with("# Two"));
    }

    #[test]
    fn test_tilde_fence_is_tracked() {
        let doc = "# One\n\n~~~\n---\n~~~\n\n---\n\n# Two\n";
        assert_eq!(split_slides(doc).len(), 2);
    }

    #[test]
    fn test_setext_heading_is_not_a_slide_break() {
        let doc = "Heading Two\n---\n\nBody text.\n";
        let slides = split_slides(doc);
        assert_eq!(slides.len(), 1, "setext H2 split the deck: {:?}", slides);
    }

    #[test]
    fn test_plain_deck_splits() {
        let doc = "# One\n\n---\n\n# Two\n\n---\n\n# Three\n";
        let slides = split_slides(doc);
        assert_eq!(slides.len(), 3);
        assert!(slides[2].starts_with("# Three"));
    }

    #[test]
    fn test_no_breaks_is_one_slide() {
        assert_eq!(split_slides("# Only\n\nText.\n").len(), 1);
    }

    #[test]
    fn test_star_and_underscore_are_not_slide_breaks() {
        // CommonMark thematic breaks, but never slide breaks under the old
        // split("\n---\n") behavior -- keep them in-slide.
        assert_eq!(split_slides("# A\n\n***\n\n# B\n").len(), 1);
        assert_eq!(split_slides("# A\n\n___\n\n# B\n").len(), 1);
    }

    #[test]
    fn test_spaced_and_long_dash_runs_split() {
        assert_eq!(split_slides("# A\n\n- - -\n\n# B\n").len(), 2);
        assert_eq!(split_slides("# A\n\n-----\n\n# B\n").len(), 2);
    }

    #[test]
    fn test_indented_dashes_are_code_not_a_break() {
        let doc = "# A\n\n    ---\n\n# B still same slide\n";
        assert_eq!(split_slides(doc).len(), 1);
    }
}
