use super::Style;
use super::color::Color;
use crate::cli::ThemeName;

/// Theme maps markdown elements to styles.
pub struct Theme {
    pub h1: Style,
    pub h2: Style,
    pub h3: Style,
    pub h4: Style,
    pub h5: Style,
    pub h6: Style,
    pub paragraph: Style,
    pub bold: Style,
    pub italic: Style,
    pub strikethrough: Style,
    pub inline_code: Style,
    pub code_border: Style,
    pub code_lang_label: Style,
    pub code_text: Style,
    pub blockquote_bar: Style,
    pub blockquote_text: Style,
    pub link_text: Style,
    pub link_url: Style,
    pub image_text: Style,
    pub list_bullet: Style,
    pub task_done: Style,
    pub task_undone: Style,
    pub table_border: Style,
    pub table_header: Style,
    pub table_cell: Style,
    pub hr: Style,
    pub footnote_ref: Style,
    pub alert_note: Style,
    pub alert_tip: Style,
    pub alert_important: Style,
    pub alert_warning: Style,
    pub alert_caution: Style,
    pub heading_rule: Style,
}

impl Theme {
    pub fn from_name(name: &ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self::dark(),
            ThemeName::Light => Self::light(),
        }
    }

    pub fn dark() -> Self {
        Theme {
            h1: Style {
                fg: Some(Color::Cyan),
                bold: true,
                ..Default::default()
            },
            h2: Style {
                fg: Some(Color::Green),
                bold: true,
                ..Default::default()
            },
            h3: Style {
                fg: Some(Color::Yellow),
                bold: true,
                ..Default::default()
            },
            h4: Style {
                fg: Some(Color::Blue),
                bold: true,
                ..Default::default()
            },
            h5: Style {
                fg: Some(Color::Magenta),
                bold: true,
                ..Default::default()
            },
            h6: Style {
                fg: Some(Color::DarkGrey),
                bold: true,
                ..Default::default()
            },
            paragraph: Style::default(),
            bold: Style {
                bold: true,
                ..Default::default()
            },
            italic: Style {
                italic: true,
                ..Default::default()
            },
            strikethrough: Style {
                strikethrough: true,
                ..Default::default()
            },
            inline_code: Style {
                fg: Some(Color::White),
                bg: Some(Color::Rgb(60, 60, 60)),
                ..Default::default()
            },
            code_border: Style {
                fg: Some(Color::DarkGrey),
                ..Default::default()
            },
            code_lang_label: Style {
                fg: Some(Color::DarkGrey),
                italic: true,
                ..Default::default()
            },
            code_text: Style::default(),
            blockquote_bar: Style {
                fg: Some(Color::DarkCyan),
                ..Default::default()
            },
            blockquote_text: Style {
                fg: Some(Color::Grey),
                italic: true,
                ..Default::default()
            },
            link_text: Style {
                fg: Some(Color::Blue),
                underline: true,
                ..Default::default()
            },
            link_url: Style {
                fg: Some(Color::DarkGrey),
                ..Default::default()
            },
            image_text: Style {
                fg: Some(Color::Magenta),
                ..Default::default()
            },
            list_bullet: Style {
                fg: Some(Color::Cyan),
                ..Default::default()
            },
            task_done: Style {
                fg: Some(Color::Green),
                ..Default::default()
            },
            task_undone: Style {
                fg: Some(Color::Red),
                ..Default::default()
            },
            table_border: Style {
                fg: Some(Color::DarkGrey),
                ..Default::default()
            },
            table_header: Style {
                fg: Some(Color::Cyan),
                bold: true,
                ..Default::default()
            },
            table_cell: Style::default(),
            hr: Style {
                fg: Some(Color::DarkGrey),
                ..Default::default()
            },
            footnote_ref: Style {
                fg: Some(Color::Cyan),
                ..Default::default()
            },
            alert_note: Style {
                fg: Some(Color::Blue),
                bold: true,
                ..Default::default()
            },
            alert_tip: Style {
                fg: Some(Color::Green),
                bold: true,
                ..Default::default()
            },
            alert_important: Style {
                fg: Some(Color::Magenta),
                bold: true,
                ..Default::default()
            },
            alert_warning: Style {
                fg: Some(Color::Yellow),
                bold: true,
                ..Default::default()
            },
            alert_caution: Style {
                fg: Some(Color::Red),
                bold: true,
                ..Default::default()
            },
            heading_rule: Style {
                fg: Some(Color::DarkGrey),
                ..Default::default()
            },
        }
    }

    pub fn light() -> Self {
        Theme {
            h1: Style {
                fg: Some(Color::DarkCyan),
                bold: true,
                ..Default::default()
            },
            h2: Style {
                fg: Some(Color::DarkGreen),
                bold: true,
                ..Default::default()
            },
            h3: Style {
                fg: Some(Color::DarkYellow),
                bold: true,
                ..Default::default()
            },
            h4: Style {
                fg: Some(Color::DarkBlue),
                bold: true,
                ..Default::default()
            },
            h5: Style {
                fg: Some(Color::DarkMagenta),
                bold: true,
                ..Default::default()
            },
            h6: Style {
                fg: Some(Color::Grey),
                bold: true,
                ..Default::default()
            },
            paragraph: Style::default(),
            bold: Style {
                bold: true,
                ..Default::default()
            },
            italic: Style {
                italic: true,
                ..Default::default()
            },
            strikethrough: Style {
                strikethrough: true,
                ..Default::default()
            },
            inline_code: Style {
                fg: Some(Color::Black),
                bg: Some(Color::Rgb(220, 220, 220)),
                ..Default::default()
            },
            code_border: Style {
                fg: Some(Color::Grey),
                ..Default::default()
            },
            code_lang_label: Style {
                fg: Some(Color::Grey),
                italic: true,
                ..Default::default()
            },
            code_text: Style::default(),
            blockquote_bar: Style {
                fg: Some(Color::DarkCyan),
                ..Default::default()
            },
            blockquote_text: Style {
                fg: Some(Color::DarkGrey),
                italic: true,
                ..Default::default()
            },
            link_text: Style {
                fg: Some(Color::DarkBlue),
                underline: true,
                ..Default::default()
            },
            link_url: Style {
                fg: Some(Color::Grey),
                ..Default::default()
            },
            image_text: Style {
                fg: Some(Color::DarkMagenta),
                ..Default::default()
            },
            list_bullet: Style {
                fg: Some(Color::DarkCyan),
                ..Default::default()
            },
            task_done: Style {
                fg: Some(Color::DarkGreen),
                ..Default::default()
            },
            task_undone: Style {
                fg: Some(Color::DarkRed),
                ..Default::default()
            },
            table_border: Style {
                fg: Some(Color::Grey),
                ..Default::default()
            },
            table_header: Style {
                fg: Some(Color::DarkCyan),
                bold: true,
                ..Default::default()
            },
            table_cell: Style::default(),
            hr: Style {
                fg: Some(Color::Grey),
                ..Default::default()
            },
            footnote_ref: Style {
                fg: Some(Color::DarkCyan),
                ..Default::default()
            },
            alert_note: Style {
                fg: Some(Color::DarkBlue),
                bold: true,
                ..Default::default()
            },
            alert_tip: Style {
                fg: Some(Color::DarkGreen),
                bold: true,
                ..Default::default()
            },
            alert_important: Style {
                fg: Some(Color::DarkMagenta),
                bold: true,
                ..Default::default()
            },
            alert_warning: Style {
                fg: Some(Color::DarkYellow),
                bold: true,
                ..Default::default()
            },
            alert_caution: Style {
                fg: Some(Color::DarkRed),
                bold: true,
                ..Default::default()
            },
            heading_rule: Style {
                fg: Some(Color::Grey),
                ..Default::default()
            },
        }
    }
}
