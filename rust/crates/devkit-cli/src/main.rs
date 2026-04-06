use clap::{Parser, Subcommand};

/// devkit-rs: repo-agnostic AI-assisted development toolkit
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable brief output mode
    #[arg(long, global = true)]
    pub brief: bool,

    /// Measure and print execution time
    #[arg(long, global = true)]
    pub time: bool,

    /// Print execution time as JSON
    #[arg(long, global = true)]
    pub time_json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show directory tree
    Tree,
    
    /// Operations on code blocks
    Block {
        #[command(subcommand)]
        command: BlockCommands,
    },

    /// Operations on markdown sections
    Md {
        #[command(subcommand)]
        command: MdCommands,
    },

    /// Patch diagnostics and application
    Patch {
        #[command(subcommand)]
        command: PatchCommands,
    },

    /// Document extraction
    Doc {
        #[command(subcommand)]
        command: DocCommands,
    },

    /// Git helper commands
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum BlockCommands {
    Outline,
    Context,
    Extract,
    Replace,
}

#[derive(Subcommand, Debug)]
pub enum MdCommands {
    AppendSection,
    ReplaceSection,
    EnsureSection,
    AppendBullet,
}

#[derive(Subcommand, Debug)]
pub enum PatchCommands {
    Diagnose,
    Apply,
}

#[derive(Subcommand, Debug)]
pub enum DocCommands {
    ImplNote,
    BenchmarkNote,
}

#[derive(Subcommand, Debug)]
pub enum GitCommands {
    CommitMessage,
    PrBody,
    SafePush,
}

fn main() {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Tree => println!("tree: Not implemented yet"),
        Commands::Block { command } => println!("block {:?}: Not implemented yet", command),
        Commands::Md { command } => println!("md {:?}: Not implemented yet", command),
        Commands::Patch { command } => println!("patch {:?}: Not implemented yet", command),
        Commands::Doc { command } => println!("doc {:?}: Not implemented yet", command),
        Commands::Git { command } => println!("git {:?}: Not implemented yet", command),
    }

    if cli.brief {
        println!("(brief mode enabled)");
    }
}
