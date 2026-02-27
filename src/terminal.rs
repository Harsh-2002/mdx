use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageProtocol {
    None,
    ITerm2,
    Kitty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorLevel {
    None,
    Basic,   // 16 colors
    Ansi256, // 256 colors
    TrueColor,
}

#[derive(Debug, Clone)]
pub struct TerminalInfo {
    pub width: u16,
    pub color_level: ColorLevel,
    pub is_tty: bool,
    pub unicode: bool,
    pub supports_osc8: bool,
    pub image_protocol: ImageProtocol,
}

impl TerminalInfo {
    pub fn detect(color_override: &crate::cli::ColorMode, width_override: Option<u16>) -> Self {
        let is_tty = std::io::stdout().is_terminal();

        let width = width_override.unwrap_or_else(|| {
            terminal_size::terminal_size()
                .map(|(w, _)| w.0)
                .unwrap_or(80)
        });

        let color_level = match color_override {
            crate::cli::ColorMode::Always => {
                // Force at least TrueColor when --color=always
                let detected = detect_color_level_from_env();
                if detected == ColorLevel::None {
                    ColorLevel::TrueColor
                } else {
                    detected
                }
            }
            crate::cli::ColorMode::Never => ColorLevel::None,
            crate::cli::ColorMode::Auto => {
                if !is_tty {
                    ColorLevel::None
                } else {
                    detect_color_level_from_env()
                }
            }
        };

        let unicode = detect_unicode();
        let supports_osc8 = match color_override {
            crate::cli::ColorMode::Always => true,
            _ => is_tty && color_level >= ColorLevel::Basic,
        };

        let image_protocol = if is_tty {
            detect_image_protocol()
        } else {
            ImageProtocol::None
        };

        TerminalInfo {
            width,
            color_level,
            is_tty,
            unicode,
            supports_osc8,
            image_protocol,
        }
    }
}

fn detect_color_level_from_env() -> ColorLevel {
    // NO_COLOR takes precedence
    if let Ok(val) = std::env::var("NO_COLOR")
        && !val.is_empty()
    {
        return ColorLevel::None;
    }

    // TERM=dumb
    if let Ok(term) = std::env::var("TERM")
        && term == "dumb"
    {
        return ColorLevel::None;
    }

    // Use supports-color crate for detection
    if let Some(level) = supports_color::on(supports_color::Stream::Stdout) {
        if level.has_16m {
            ColorLevel::TrueColor
        } else if level.has_256 {
            ColorLevel::Ansi256
        } else if level.has_basic {
            ColorLevel::Basic
        } else {
            ColorLevel::None
        }
    } else {
        ColorLevel::None
    }
}

fn detect_image_protocol() -> ImageProtocol {
    // Check TERM_PROGRAM for iTerm2 and WezTerm (both support iTerm2 protocol)
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            "iTerm.app" | "WezTerm" => return ImageProtocol::ITerm2,
            _ => {}
        }
    }
    // Check for Kitty
    if std::env::var("KITTY_PID").is_ok() {
        return ImageProtocol::Kitty;
    }
    ImageProtocol::None
}

fn detect_unicode() -> bool {
    for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(val) = std::env::var(var)
            && !val.is_empty()
        {
            let upper = val.to_uppercase();
            return upper.contains("UTF");
        }
    }
    // Default to true on macOS
    cfg!(target_os = "macos")
}
