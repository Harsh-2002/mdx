use clap::{Parser, Subcommand, ValueHint};

#[derive(Parser, Debug)]
#[command(
    name = "mdx",
    version,
    about = "A fast terminal markdown renderer and toolchain",
    subcommand_help_heading = "Commands",
    after_help = "Examples:\n  mdx README.md                          Render in terminal\n  mdx serve .                            Live preview in browser\n  mdx fetch https://example.com          Fetch web page as markdown\n  mdx stats README.md                    Show word count & stats\n  mdx fmt --check README.md              Check formatting\n  mdx export --to html README.md         Export to HTML\n  mdx export --to pdf README.md          Export to PDF\n  mdx export --to epub README.md         Export to EPUB e-book\n  mdx publish ./blog --out ./dist        Generate static site\n  mdx update                             Update to latest version\n  mdx completions install                Install shell completions"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Markdown file to render (reads stdin if omitted)
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: Option<String>,

    /// Output width in columns (default: terminal width)
    ///
    /// Accepted before or after any subcommand. Honored where output goes to
    /// the terminal.
    #[arg(short, long, global = true, help_heading = "Presentation")]
    pub width: Option<u16>,

    /// Pipe output through less -R (top level and `mdx fetch`)
    ///
    /// Not global: `mdx serve` owns `-p` for `--port`, and a global short is
    /// pushed into every subcommand, which clap rejects at build time.
    #[arg(short, long, help_heading = "Presentation")]
    pub pager: bool,

    /// Color mode: auto, always, never
    ///
    /// Accepted before or after any subcommand.
    #[arg(
        long,
        global = true,
        default_value = "auto",
        help_heading = "Presentation"
    )]
    pub color: ColorMode,

    /// Theme: dark (default), light
    ///
    /// Accepted before or after any subcommand. Honored by the terminal,
    /// serve, watch, present, publish and `export --to html`.
    #[arg(
        long,
        global = true,
        default_value = "dark",
        help_heading = "Presentation"
    )]
    pub theme: ThemeName,

    /// Plain text output (no ANSI, no box-drawing, no fancy bullets)
    ///
    /// Accepted before or after any subcommand.
    #[arg(long, global = true, help_heading = "Presentation")]
    pub plain: bool,

    /// Syntax highlighting theme for code blocks
    ///
    /// Accepted before or after any subcommand. Honored by every target that
    /// highlights code.
    #[arg(
        long,
        global = true,
        default_value = crate::options::DEFAULT_SYNTAX_THEME,
        help_heading = "Presentation"
    )]
    pub syntax_theme: String,

    /// Custom CSS file to inject into HTML output (serve, export --to html, publish)
    #[arg(
        long,
        global = true,
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        help_heading = "Presentation"
    )]
    pub css: Option<String>,

    /// List available syntax highlighting themes and exit
    #[arg(long)]
    pub list_syntax_themes: bool,

    /// Generate man page
    #[arg(long)]
    pub generate_man: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Preview markdown in the browser with live reload
    #[cfg(feature = "serve")]
    Serve(ServeArgs),

    /// Watch a file and re-render on changes
    #[cfg(feature = "watch")]
    Watch(WatchArgs),

    /// Present markdown as slides in the terminal
    #[cfg(feature = "watch")]
    Present(PresentArgs),

    /// Generate a table of contents
    Toc(TocArgs),

    /// Show document statistics (words, lines, headings, etc.)
    Stats(StatsArgs),

    /// Format/prettify markdown
    Fmt(FmtArgs),

    /// Check markdown for common issues
    Lint(LintArgs),

    /// Compare two markdown files with colored diff
    Diff(DiffArgs),

    /// Export markdown to another format (html, pdf, epub, json, txt)
    Export(ExportArgs),

    /// Generate a static site from a directory of markdown files
    Publish(PublishArgs),

    /// Search markdown files for a query
    Search(SearchArgs),

    /// Fetch a web page and extract its content as markdown
    #[cfg(feature = "url")]
    Fetch(FetchArgs),

    /// Update mdx to the latest version
    #[cfg(feature = "url")]
    Update,

    /// Generate or install shell completions
    Completions(CompletionsArgs),
}

#[derive(clap::Args, Debug)]
pub struct CompletionsArgs {
    /// Shell name (bash, zsh, fish, powershell) or "install" to auto-install
    pub shell_or_action: String,
}

#[cfg(feature = "serve")]
#[derive(clap::Args, Debug)]
pub struct ServeArgs {
    /// Markdown file(s) or directory to preview (reads stdin if omitted or "-")
    #[arg(value_hint = ValueHint::AnyPath)]
    pub files: Vec<String>,

    /// Port (default: random available port)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Address to bind to (default: loopback only, reachable from this machine)
    ///
    /// Use --host 0.0.0.0 to reach the preview from other devices on your
    /// network. The preview server has no authentication: anyone who can reach
    /// the port can read these files, overwrite them (PUT /source), create new
    /// ones (POST /create) and upload files into ./assets (POST /upload).
    /// Accepts an IP address or "localhost".
    #[arg(long, default_value = "127.0.0.1", value_name = "ADDR")]
    pub host: String,

    /// Render raw HTML embedded in the markdown (unsafe)
    ///
    /// Raw HTML is dropped by default, so a `<script>` tag in a downloaded,
    /// cloned or agent-written document cannot run in your browser. Turn this
    /// on only for documents you trust; it also re-enables javascript: and
    /// data: links.
    #[arg(long)]
    pub unsafe_html: bool,
}

#[cfg(feature = "watch")]
#[derive(clap::Args, Debug)]
pub struct WatchArgs {
    /// Markdown file to watch
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: String,
}

#[derive(clap::Args, Debug)]
pub struct TocArgs {
    /// Markdown file to generate TOC from
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: String,

    /// Maximum heading depth to include (1-6)
    #[arg(long, default_value = "3")]
    pub depth: u8,
}

#[cfg(feature = "watch")]
#[derive(clap::Args, Debug)]
pub struct PresentArgs {
    /// Markdown file to present
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: String,
}

#[derive(clap::Args, Debug)]
pub struct StatsArgs {
    /// Markdown file (reads stdin if omitted)
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: Option<String>,
}

#[derive(clap::Args, Debug)]
#[command(
    after_help = "Examples:\n  mdx fmt README.md                Print formatted to stdout\n  mdx fmt --in-place README.md     Format file in place\n  mdx fmt --check README.md        Exit 1 if not formatted (CI)"
)]
pub struct FmtArgs {
    /// Markdown file (reads stdin if omitted)
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: Option<String>,

    /// Format file in place (overwrites the file)
    #[arg(short, long)]
    pub in_place: bool,

    /// Check if file is formatted (exit 1 if not, for CI)
    #[arg(short, long)]
    pub check: bool,
}

#[derive(clap::Args, Debug)]
pub struct LintArgs {
    /// Markdown file to lint
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: String,
}

#[derive(clap::Args, Debug)]
#[command(
    after_help = "Examples:\n  mdx diff old.md new.md           Side-by-side diff\n  mdx diff -u old.md new.md        Unified diff\n  mdx diff - new.md                Read old from stdin"
)]
pub struct DiffArgs {
    /// First file (use "-" for stdin)
    #[arg(value_hint = ValueHint::FilePath)]
    pub file_a: String,

    /// Second file
    #[arg(value_hint = ValueHint::FilePath)]
    pub file_b: String,

    /// Show unified diff instead of side-by-side
    #[arg(long, short)]
    pub unified: bool,
}

#[derive(clap::Args, Debug)]
#[command(
    after_help = "Examples:\n  mdx export --to html README.md          Standalone HTML page\n  mdx export --to pdf README.md           PDF document\n  mdx export --to pdf -o out.pdf file.md  PDF with custom output path\n  mdx export --to epub README.md          EPUB e-book\n  mdx export --to json README.md          AST as JSON\n  mdx export --to txt README.md           Plain text (strip formatting)"
)]
pub struct ExportArgs {
    /// Markdown file (reads stdin if omitted)
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: Option<String>,

    /// Output format
    #[arg(long, value_parser = ["html", "json", "txt", "pdf", "epub"])]
    pub to: String,

    /// Output file path (pdf/epub default to the input name with a new extension)
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub output: Option<String>,

    /// Allow uploading mermaid source to kroki.io when mmdc is missing (pdf only)
    #[arg(long)]
    pub allow_remote_render: bool,
}

#[derive(clap::Args, Debug)]
#[command(
    after_help = "Front matter (optional):\n  ---\n  title: My Post\n  date: 2024-01-15\n  tags: rust, cli\n  draft: true\n  ---"
)]
pub struct PublishArgs {
    /// Directory containing markdown files
    #[arg(value_hint = ValueHint::DirPath)]
    pub dir: String,

    /// Output directory for the generated site
    #[arg(long, short, default_value = "dist", value_hint = ValueHint::DirPath)]
    pub out: String,

    /// Render raw HTML embedded in the markdown (unsafe)
    ///
    /// Raw HTML is dropped by default. A published site runs on a real origin,
    /// where a `<script>` from a post you did not write is worse than in a
    /// local preview. Turn this on only for content you trust.
    #[arg(long)]
    pub unsafe_html: bool,
}

#[derive(clap::Args, Debug)]
#[command(
    after_help = "Examples:\n  mdx search \"rust async\" .            Search current directory\n  mdx search \"BM25\" docs/             Search recursively\n  mdx search --tag rust \"ownership\"    Filter by front matter tag\n  mdx search -n 5 \"error\" .            Top 5 results\n  mdx search -l \"query\" .              List matching files only"
)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Files or directories to search (default: current directory)
    #[arg(value_hint = ValueHint::AnyPath)]
    pub paths: Vec<String>,

    /// Max results (default: 10)
    #[arg(short = 'n', long, default_value = "10")]
    pub limit: usize,

    /// Filter by front matter tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Show file paths only (for piping)
    #[arg(short = 'l', long)]
    pub files_only: bool,
}

#[cfg(feature = "url")]
#[derive(clap::Args, Debug)]
pub struct FetchArgs {
    /// URL to fetch
    pub url: String,
    /// Save output to a file instead of stdout
    #[arg(short, long)]
    pub output: Option<String>,
    /// Convert full HTML to markdown (skip readability extraction)
    #[arg(long)]
    pub raw: bool,
    /// Include YAML front matter with title, author, date, source URL
    #[arg(long)]
    pub metadata: bool,
    /// Show estimated token count
    #[arg(long)]
    pub tokens: bool,

    /// Pipe output through less -R (same as the top-level -p)
    #[arg(short = 'p', long)]
    pub pager: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ThemeName {
    Dark,
    Light,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_is_well_formed() {
        Args::command().debug_assert();
    }

    /// clap silently skips propagating a global into a subcommand that
    /// redeclares its id, so a reintroduced ServeArgs::css would parse and
    /// then be ignored. Nothing else would report that.
    #[cfg(feature = "serve")]
    #[test]
    fn serve_css_is_the_global_css() {
        let mut cmd = Args::command();
        cmd.build();
        let serve = cmd.find_subcommand("serve").expect("serve subcommand");
        let css: Vec<_> = serve
            .get_arguments()
            .filter(|a| a.get_long() == Some("css"))
            .collect();
        assert_eq!(css.len(), 1, "exactly one --css must reach serve");
        assert!(
            css[0].is_global_set(),
            "serve must not redeclare --css: a local copy silently shadows the global"
        );
    }
}
