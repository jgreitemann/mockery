use clap::{AppSettings, Clap};

/// A tool for creating Google Mock mock class definitions based on the pure virtual member
/// functions of an interface class, and for keeping the mock up-to-date as the interface evolves.
#[derive(Clap)]
#[clap(version = "0.1.0", author = "Jonas Greitemann <jgreitemann@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct MockeryOpts {
    /// Path to the compile commands database (`compile_commands.json`). By default, we will try to
    /// find the database in the vicinity of the provided source file.
    #[clap(short, long)]
    pub compile_commands: Option<String>,

    /// The "search radius" around the provided source file in which the compile commands database
    /// is searched for if the path is not explicitly provided. Relative paths of up to this number
    /// of levels (either up or down), relative to the source file, are considered.
    #[clap(short = 'r', long, default_value = "2")]
    pub search_radius: usize,

    /// A level of verbosity; can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Create(CreateOpts),
    Update(UpdateOpts),
    Dump(DumpOpts),
}

/// Create a mock class definition from scratch based on an interface class.
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct CreateOpts {
    /// Path to a translation unit (*.cpp) which includes the interface that is to be mocked.
    pub interface_source: String,

    /// Name of the interface class that is to be mocked. The default is inferred from the filename
    /// of the source translation unit.
    #[clap(short, long)]
    pub interface: Option<String>,

    /// Name which is given to the resulting mock class. By default, the interface class's name is
    /// suffixed with `Mock`.
    #[clap(short, long)]
    pub mock: Option<String>,

    /// Path to the file which the mock class definition should be written to. If the file already
    /// exists, it will be overwritten!
    #[clap(short, long)]
    pub output: Option<String>,

    /// Force the mock class definition to be written to stdout. This is the default behavior in the
    /// absence of `--output`; `--stdout` can be used to retain this behavior even in the presence
    /// of `--output`.
    #[clap(long)]
    pub stdout: bool,
}

/// Modify an existing mock class definition to mirror changes to the underlying interface class.
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct UpdateOpts {
    /// Path to a translation unit (*.cpp) which includes the mock class which needs to be updated.
    pub mock_source: String,

    /// Name of the mock class which needs to be updated. The default is inferred from the filename
    /// of the source translation unit.
    #[clap(short, long)]
    pub mock: Option<String>,

    /// Do not actually modify any source files. Useful in combination with `--diff`. This is *not*
    /// the default behavior, unless `--patch` is used.
    #[clap(long)]
    pub dry_run: bool,

    /// Print a diff of the changes to the mock class definition to stdout. The changes are still
    /// applied to the source files, unless `--dry-run` is used.
    #[clap(short, long)]
    pub diff: bool,

    /// Format a patch of the changes to the mock class definition and writes it to the specified
    /// file. Implies `--dry-run`.
    #[clap(short, long)]
    pub patch: Option<String>,
}

/// Dump the AST for the specified source file or class.
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct DumpOpts {
    /// Path to a translation unit (*.cpp).
    pub source: String,

    /// Name of the class whose AST should be dumped. If none is specified, the AST of the whole
    /// translation unit is dumped instead.
    #[clap(long)]
    pub class: Option<String>,
}
