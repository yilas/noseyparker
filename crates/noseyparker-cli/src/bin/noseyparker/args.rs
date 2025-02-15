use anyhow::{Context, Result};
use clap::{crate_description, crate_version, ArgAction, Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use url::Url;

use noseyparker::git_url::GitUrl;

fn get_long_version() -> &'static str {
    concat!(
        crate_version!(),
        "\n",
        "\n", "Build Configuration:",
        "\n",
        "\n", "    Build Timestamp:    ", env!("VERGEN_BUILD_TIMESTAMP"),
        "\n",
        "\n", "    Commit Timestamp:   ", env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
        "\n", "    Commit Branch:      ", env!("VERGEN_GIT_BRANCH"),
        "\n", "    Commit SHA:         ", env!("VERGEN_GIT_SHA"),
        "\n",
        "\n", "    Cargo Features:     ", env!("VERGEN_CARGO_FEATURES"),
        "\n", "    Debug:              ", env!("VERGEN_CARGO_DEBUG"),
        "\n", "    Optimization:       ", env!("VERGEN_CARGO_OPT_LEVEL"),
        "\n", "    Target Triple:      ", env!("VERGEN_CARGO_TARGET_TRIPLE"),
        "\n",
        "\n", "Build System:",
        "\n",
        "\n", "    OS:                 ", env!("VERGEN_SYSINFO_NAME"),
        "\n", "    OS Version:         ", env!("VERGEN_SYSINFO_OS_VERSION"),
        "\n",
        "\n", "    CPU Vendor:         ", env!("VERGEN_SYSINFO_CPU_VENDOR"),
        "\n", "    CPU Brand:          ", env!("VERGEN_SYSINFO_CPU_BRAND"),
        "\n", "    CPU Cores:          ", env!("VERGEN_SYSINFO_CPU_CORE_COUNT"),
        "\n",
        "\n", "    rustc Version:      ", env!("VERGEN_RUSTC_SEMVER"),
        "\n", "    rustc Channel:      ", env!("VERGEN_RUSTC_CHANNEL"),
        "\n", "    rustc Host Triple:  ", env!("VERGEN_RUSTC_HOST_TRIPLE"),
        "\n", "    rustc Commit Date:  ", env!("VERGEN_RUSTC_COMMIT_DATE"),
        "\n", "    rustc Commit SHA:   ", env!("VERGEN_RUSTC_COMMIT_HASH"),
        "\n", "    rustc LLVM Version: ", env!("VERGEN_RUSTC_LLVM_VERSION"),
    )
}

// -----------------------------------------------------------------------------
// command-line args
// -----------------------------------------------------------------------------
#[derive(Parser, Debug)]
#[command(
    name("noseyparker"),

    author,   // retrieved from Cargo.toml `authors`
    version,  // retrieved from Cargo.toml `version`
    about,    // retrieved from Cargo.toml `description`

    long_version = get_long_version(),

    // FIXME: add longer comment description (will be shown with `--help`)
    long_about = concat!(
        crate_description!(),
    ),
)]
#[deny(missing_docs)]
/// Find secrets and sensitive information in textual data
pub struct CommandLineArgs {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    // FIXME: suppress from showing long help in subcommand help; only show on top-level `help`
    pub global_args: GlobalArgs,
}

impl CommandLineArgs {
    pub fn parse_args() -> Self {
        let mut s = Self::parse();

        // If `NO_COLOR` is set in the environment, disable colored output
        //
        // https://no-color.org/
        if std::env::var("NO_COLOR").is_ok() {
            s.global_args.color = Mode::Never
        }

        s
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scan content for secrets
    ///
    /// This command uses regex-based rules to identify hardcoded secrets and other potentially sensitive information in textual content (or in inputs that can have textual content extracted from them).
    ///
    /// The findings from scanning are recorded into a datastore. The recorded findings can later
    /// be reported in several formats using the `summarize` and `report` commands.
    ///
    /// Several types of inputs can be specified:
    ///
    /// - Positional input arguments can be either files or directories.
    ///   Files are scanned directly; directories are recursively enumerated and scanned.
    ///   Any directories encountered that are Git repositories will have their entire history scanned.
    ///
    /// - A Git repository URL can be specified with the `--git-repo=URL` argument.
    ///   This will cause Nosey Parker to clone that repository to its datastore and scan its history.
    ///
    /// - A GitHub user can be specified with the `--github-user=NAME` argument.
    ///   This will cause Nosey Parker to enumerate accessible repositories belonging to that user, clone them to its datastore, and scan their entire history.
    ///
    /// - A GitHub organization can be specified with the `--github-org=NAME` argument.
    ///   This will cause Nosey Parker to enumerate accessible repositories belonging to that organization, clone them to its datastore, and scan their entire history.
    ///
    /// The `git` binary on the PATH is used to clone any required Git repositories.
    /// It is careful invoked to avoid using any system-wide or user-specific configuration.
    ///
    /// By default, when cloning repositories from GitHub or enumerating GitHub users or organizations, unauthenticated access is used.
    /// An optional personal access token can be specified using the `NP_GITHUB_TOKEN` environment variable.
    /// Using a personal access token gives higher rate limits and may make additional content accessible.
    #[command(display_order = 1)]
    Scan(ScanArgs),

    /// Summarize scan findings
    #[command(display_order = 2, alias = "summarise")]
    Summarize(SummarizeArgs),

    /// Report detailed scan findings
    #[command(display_order = 3)]
    Report(ReportArgs),

    /// Interact with GitHub
    ///
    /// By default, unauthenticated access is used.
    /// An optional personal access token can be specified using the `NP_GITHUB_TOKEN` environment variable.
    /// Using a personal access token gives higher rate limits and may make additional content accessible.
    #[command(display_order = 4, name = "github")]
    GitHub(GitHubArgs),

    #[command(display_order = 30)]
    /// Manage datastores
    Datastore(DatastoreArgs),

    #[command(display_order = 30)]
    /// Manage rules
    Rules(RulesArgs),
}

// -----------------------------------------------------------------------------
// global options
// -----------------------------------------------------------------------------
#[derive(Args, Debug)]
#[command(next_help_heading = "Global Options")]
pub struct GlobalArgs {
    /// Enable verbose output
    ///
    /// This can be repeated up to 3 times to enable successively more output.
    #[arg(global=true, long, short, action=ArgAction::Count)]
    pub verbose: u8,

    /// Enable or disable colored output
    ///
    /// When this is "auto", colors are enabled when stdout is a tty.
    ///
    /// If the `NO_COLOR` environment variable is set, it takes precedence and is equivalent to `--color=never`.
    #[arg(global=true, long, default_value_t=Mode::Auto, value_name="MODE")]
    pub color: Mode,

    /// Enable or disable progress bars
    ///
    /// When this is "auto", progress bars are enabled when stderr is a tty.
    #[arg(global=true, long, default_value_t=Mode::Auto, value_name="MODE")]
    pub progress: Mode,

    #[command(flatten)]
    pub advanced: AdvancedArgs,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Advanced Global Options")]
/// These are advanced options that should not need to be used in normal circumstances.
pub struct AdvancedArgs {
    /// Set the rlimit for number of open files to LIMIT
    ///
    /// This should not need to be changed from the default unless you run into crashes from
    /// running out of file descriptors.
    #[arg(hide_short_help=true, global=true, long, default_value_t=16384, value_name="LIMIT")]
    pub rlimit_nofile: u64,

    /// Enable or disable backtraces on panic
    ///
    /// This has the effect of setting the `RUST_BACKTRACE` environment variable to 1.
    #[arg(hide_short_help=true, global=true, long, default_value_t=true, action=ArgAction::Set, value_name="BOOL")]
    pub enable_backtraces: bool,
}

impl GlobalArgs {
    pub fn use_color(&self) -> bool {
        match self.color {
            Mode::Never => false,
            Mode::Always => true,
            Mode::Auto => atty::is(atty::Stream::Stdout),
        }
    }

    pub fn use_progress(&self) -> bool {
        match self.progress {
            Mode::Never => false,
            Mode::Always => true,
            Mode::Auto => atty::is(atty::Stream::Stderr),
        }
    }
}

/// A generic auto/never/always mode value
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    Auto,
    Never,
    Always,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Mode::Auto => "auto",
            Mode::Never => "never",
            Mode::Always => "always",
        };
        write!(f, "{s}")
    }
}

// -----------------------------------------------------------------------------
// `github` command
// -----------------------------------------------------------------------------
#[derive(Args, Debug)]
pub struct GitHubArgs {
    #[command(subcommand)]
    pub command: GitHubCommand,

    /// Use the specified URL for GitHub API access
    ///
    /// If accessing a GitHub Enterprise Server instance, this value should be the entire base URL
    /// include the `api/v3` portion, e.g., `https://github.example.com/api/v3`.
    #[arg(
        long,
        value_name = "URL",
        default_value_t = Url::parse("https://api.github.com").expect("default API url should parse"),
        visible_alias="api-url"
    )]
    pub github_api_url: Url,
}

#[derive(Subcommand, Debug)]
pub enum GitHubCommand {
    /// Interact with GitHub repositories
    #[command(subcommand)]
    Repos(GitHubReposCommand),
}

#[derive(Subcommand, Debug)]
pub enum GitHubReposCommand {
    /// List repositories belonging to a specific user or organization
    List(GitHubReposListArgs),
}

#[derive(Args, Debug)]
pub struct GitHubReposListArgs {
    #[command(flatten)]
    pub repo_specifiers: GitHubRepoSpecifiers,

    #[command(flatten)]
    pub output_args: OutputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GitHubRepoSpecifiers {
    /// Select repositories belonging to the specified user
    ///
    /// This option can be repeated.
    #[arg(long)]
    pub user: Vec<String>,

    /// Select repositories belonging to the specified organization
    ///
    /// This option can be repeated.
    #[arg(long, visible_alias = "org")]
    pub organization: Vec<String>,
}

impl GitHubRepoSpecifiers {
    pub fn is_empty(&self) -> bool {
        self.user.is_empty() && self.organization.is_empty()
    }
}

// -----------------------------------------------------------------------------
// `rules` command
// -----------------------------------------------------------------------------
#[derive(Args, Debug)]
pub struct RulesArgs {
    #[command(subcommand)]
    pub command: RulesCommand,
}

#[derive(Subcommand, Debug)]
pub enum RulesCommand {
    /// Check rules for problems
    ///
    /// If errors are detected or if warnings are detected and `--warnings-as-errors` is specified, the program will exit with a nonzero exit code.
    Check(RulesCheckArgs),
}

#[derive(Args, Debug)]
pub struct RulesCheckArgs {
    #[arg(long, short = 'W')]
    /// Treat warnings as errors
    pub warnings_as_errors: bool,

    #[arg(num_args(1..), required(true))]
    /// Files or directories to check
    pub inputs: Vec<PathBuf>,
}

// -----------------------------------------------------------------------------
// `datastore` command
// -----------------------------------------------------------------------------
#[derive(Args, Debug)]
pub struct DatastoreArgs {
    #[command(subcommand)]
    pub command: DatastoreCommand,
}

#[derive(Subcommand, Debug)]
pub enum DatastoreCommand {
    /// Initialize a new datastore
    Init(DatastoreInitArgs),
}

#[derive(Args, Debug)]
pub struct DatastoreInitArgs {
    #[arg(long, short, value_name = "PATH", env("NP_DATASTORE"))]
    /// Initialize the datastore at specified path
    pub datastore: PathBuf,
}

fn get_parallelism() -> usize {
    match std::thread::available_parallelism() {
        Err(_e) => 1,
        Ok(v) => v.into(),
    }
}

// -----------------------------------------------------------------------------
// `scan` command
// -----------------------------------------------------------------------------
/// Arguments for the `scan` command
#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Use the specified datastore
    ///
    /// The datastore will be created if it does not exist.
    #[arg(long, short, value_name = "PATH", env("NP_DATASTORE"))]
    // FIXME: choose a good default value for this?
    pub datastore: PathBuf,

    /// Use N parallel scanning jobs
    #[arg(long("jobs"), short('j'), value_name="N", default_value_t=get_parallelism())]
    pub num_jobs: usize,

    /// Include up to the specified number of bytes before and after each match
    ///
    /// The default value typically gives between 4 and 7 lines of context before and after each
    /// match.
    #[arg(long, value_name="BYTES", default_value_t=256)]
    pub snippet_length: usize,

    /// Use custom rules from the specified file or directory
    ///
    /// The paths can be either files or directories.
    /// Directories are recursively walked and all discovered rule files will be loaded.
    ///
    /// This option can be repeated.
    #[arg(long, short, value_name = "PATH")]
    pub rules: Vec<PathBuf>,

    #[command(flatten)]
    pub input_specifier_args: InputSpecifierArgs,

    #[command(flatten)]
    pub content_filtering_args: ContentFilteringArgs,
}

/// The mode to use for cloning a Git repository
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GitCloneMode {
    /// Match the behavior of `git clone --bare`
    Bare,

    /// Match the behavior of `git clone --mirror`
    ///
    /// This will clone the most possible content.
    /// When cloning repositories hosted on GitHub, this mode may clone objects that come from forks.
    Mirror,
}

impl std::fmt::Display for GitCloneMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GitCloneMode::Bare => "bare",
            GitCloneMode::Mirror => "mirror",
        };
        write!(f, "{s}")
    }
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Input Specifier Options")]
pub struct InputSpecifierArgs {
    /// Scan the specified file, directory, or local Git repository
    #[arg(value_name="INPUT", required_unless_present_any(["github_user", "github_organization", "git_url"]), display_order=1)]
    pub path_inputs: Vec<PathBuf>,

    /// Clone and scan the Git repository at the specified URL
    ///
    /// Only https URLs without credentials, query parameters, or fragment identifiers are supported.
    ///
    /// This option can be repeated.
    #[arg(long, value_name = "URL", display_order = 10)]
    pub git_url: Vec<GitUrl>,

    /// Clone and scan accessible repositories belonging to the specified GitHub user
    ///
    /// This option can be repeated.
    #[arg(long, value_name = "NAME", display_order = 20)]
    pub github_user: Vec<String>,

    /// Clone and scan accessible repositories belonging to the specified GitHub organization
    ///
    /// This option can be repeated.
    #[arg(
        long,
        visible_alias = "github-org",
        value_name = "NAME",
        display_order = 20
    )]
    pub github_organization: Vec<String>,

    /// Use the specified URL for GitHub API access
    ///
    /// If accessing a GitHub Enterprise Server instance, this value should be the entire base URL
    /// include the `api/v3` portion, e.g., `https://github.example.com/api/v3`.
    #[arg(
        long,
        visible_alias = "api-url",
        value_name = "URL",
        default_value_t = Url::parse("https://api.github.com").expect("default API url should parse"),
        display_order = 30
    )]
    pub github_api_url: Url,

    /// Use the specified method for cloning Git repositories
    #[arg(long, value_name = "MODE", display_order = 40, default_value_t=GitCloneMode::Bare)]
    pub git_clone_mode: GitCloneMode,
}

/// This struct represents options to control content discovery.
#[derive(Args, Debug)]
#[command(next_help_heading = "Content Filtering Options")]
pub struct ContentFilteringArgs {
    /// Do not scan files larger than the specified size
    ///
    /// The value is parsed as a floating point literal, and hence fractional values can be supplied.
    /// A negative value means "no limit".
    /// Note that scanning requires reading the entire contents of each file into memory, so using an excessively large limit may be problematic.
    #[arg(
        long("max-file-size"),
        default_value_t = 100.0,
        value_name = "MEGABYTES"
    )]
    pub max_file_size_mb: f64,

    /// Use custom path-based ignore rules from the specified file
    ///
    /// The ignore file should contain gitignore-style rules.
    ///
    /// This option can be repeated.
    #[arg(long, short, value_name = "FILE")]
    pub ignore: Vec<PathBuf>,
    /*
    /// Do not scan files that appear to be binary
    #[arg(long)]
    pub skip_binary_files: bool,
    */
}

impl ContentFilteringArgs {
    pub fn max_file_size_bytes(&self) -> Option<u64> {
        if self.max_file_size_mb < 0.0 {
            None
        } else {
            Some((self.max_file_size_mb * 1024.0 * 1024.0) as u64)
        }
    }
}

// -----------------------------------------------------------------------------
// `summarize` command
// -----------------------------------------------------------------------------
#[derive(Args, Debug)]
pub struct SummarizeArgs {
    /// Use the specified datastore
    #[arg(long, short, value_name = "PATH", env("NP_DATASTORE"))]
    pub datastore: PathBuf,

    #[command(flatten)]
    pub output_args: OutputArgs,
}

// -----------------------------------------------------------------------------
// `report` command
// -----------------------------------------------------------------------------
#[derive(Args, Debug)]
pub struct ReportArgs {
    /// Use the specified datastore
    #[arg(long, short, value_name = "PATH", env("NP_DATASTORE"))]
    pub datastore: PathBuf,

    #[command(flatten)]
    pub output_args: OutputArgs,
}

// -----------------------------------------------------------------------------
// output options
// -----------------------------------------------------------------------------
#[derive(Args, Debug)]
#[command(next_help_heading = "Output Options")]
pub struct OutputArgs {
    /// Write output to the specified path
    ///
    /// If this argument is not provided, stdout will be used.
    #[arg(long, short, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Write output in the specified format
    // FIXME: make this optional, and if not specified, infer from the extension of the output file
    #[arg(long, short, value_name="FORMAT", default_value_t=OutputFormat::Human)]
    pub format: OutputFormat,
}

impl OutputArgs {
    /// Get a writer for the specified output destination.
    pub fn get_writer(&self) -> std::io::Result<Box<dyn std::io::Write>> {
        use std::fs::File;
        use std::io::BufWriter;

        match &self.output {
            None => Ok(Box::new(BufWriter::new(std::io::stdout()))),
            Some(p) => {
                let f = File::create(p)?;
                Ok(Box::new(BufWriter::new(f)))
            }
        }
    }
}

// -----------------------------------------------------------------------------
// output format
// -----------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// A text-based format designed for humans
    Human,

    /// Pretty-printed JSON format
    Json,

    /// JSON Lines format
    ///
    /// This is a sequence of JSON objects, one per line.
    Jsonl,

    /// SARIF format
    ///
    /// This is the Static Analysis Results Interchange Format, a standardized JSON-based format used by many tools.
    /// See the spec at <https://docs.oasis-open.org/sarif/sarif/v2.1.0/cs01/sarif-v2.1.0-cs01.html>.
    Sarif,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OutputFormat::Human => "human",
            OutputFormat::Json => "json",
            OutputFormat::Jsonl => "jsonl",
            OutputFormat::Sarif => "sarif",
        };
        write!(f, "{s}")
    }
}

// -----------------------------------------------------------------------------
// report writer
// -----------------------------------------------------------------------------
pub trait Reportable {
    fn human_format<W: std::io::Write>(&self, writer: W) -> Result<()>;
    fn json_format<W: std::io::Write>(&self, writer: W) -> Result<()>;
    fn jsonl_format<W: std::io::Write>(&self, writer: W) -> Result<()>;
    fn sarif_format<W: std::io::Write>(&self, writer: W) -> Result<()>;

    fn report(&self, output_args: &OutputArgs) -> Result<()> {
        let writer = output_args
            .get_writer()
            .context("Failed to open output destination for writing")?;

        let result = match &output_args.format {
            OutputFormat::Human => self.human_format(writer),
            OutputFormat::Json => self.json_format(writer),
            OutputFormat::Jsonl => self.jsonl_format(writer),
            OutputFormat::Sarif => self.sarif_format(writer),
        };
        match result {
            Ok(()) => Ok(()),
            Err(e) => match e.downcast_ref::<std::io::Error>() {
                // Ignore SIGPIPE errors, like those that can come from piping to `head`
                Some(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
                _ => Err(e)?,
            },
        }
    }
}
