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
    Tree {
        /// Root directory to scan (default: current directory)
        #[arg(long)]
        path: Option<String>,
        
        /// Maximum depth to descend
        #[arg(long)]
        max_depth: Option<usize>,
        
        /// Comma-separated list of extensions to include (e.g. '.py,.rs')
        #[arg(long)]
        ext: Option<String>,
        
        /// Show only directories
        #[arg(long)]
        dirs_only: bool,
        
        /// Do not read .gitignore
        #[arg(long)]
        no_gitignore: bool,
    },
    
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
    /// Extract function/class signatures
    Outline {
        file: String,
        #[arg(long)]
        imports: bool,
        #[arg(long)]
        docstrings: bool,
    },
    /// Extract function context
    Context {
        file: String,
        function: String,
        #[arg(long, default_value_t = 5)]
        margin: usize,
    },
    /// Extract an exact block
    Extract {
        file: String,
        #[arg(long, allow_hyphen_values = true)]
        lines: Option<String>,
        #[arg(long)]
        marker: Option<String>,
        #[arg(long)]
        heading: Option<String>,
        #[arg(long)]
        function: Option<String>,
    },
    /// Replace an exact block
    Replace {
        file: String,
        #[arg(long)]
        with_file: String,
        #[arg(long, allow_hyphen_values = true)]
        lines: Option<String>,
        #[arg(long)]
        marker: Option<String>,
        #[arg(long)]
        heading: Option<String>,
        #[arg(long)]
        function: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum MdCommands {
    /// Append content to a markdown section
    AppendSection {
        file: String,
        heading: String,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        with_file: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Replace the body of a markdown section
    ReplaceSection {
        file: String,
        heading: String,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        with_file: Option<String>,
        #[arg(long)]
        no_keep_heading: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Ensure a section exists
    EnsureSection {
        file: String,
        heading: String,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        with_file: Option<String>,
        #[arg(long, default_value_t = 2)]
        level: usize,
        #[arg(long)]
        after: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Append a list bullet item
    AppendBullet {
        file: String,
        heading: String,
        bullet: String,
        #[arg(long)]
        dedupe: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum PatchCommands {
    /// Diagnose patch application
    Diagnose {
        file: String,
    },
    Apply {
        file: String,
        #[arg(long)]
        reject: bool,
        #[arg(long)]
        verbose: bool,
    },
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
        Commands::Tree { path, max_depth, ext, dirs_only, no_gitignore } => {
            use std::path::PathBuf;
            let root = path.as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                
            let root = root.canonicalize().unwrap_or(root);
            if !root.is_dir() {
                eprintln!("Error: {} is not a directory.", root.display());
                std::process::exit(1);
            }

            let extra_ignore = match devkit_core::load_config(&root) {
                Ok(config) => config.encoding.ignore,
                Err(_) => Vec::new(),
            };

            let extensions = ext.as_ref().map(|e| {
                e.split(',')
                 .map(|s| s.trim().to_string())
                 .collect::<Vec<_>>()
            });

            let entry = devkit_tree::scan_tree(
                &root,
                *max_depth,
                extensions.as_deref(),
                *dirs_only,
                !*no_gitignore,
                &extra_ignore,
            );

            let mut lines = Vec::new();
            devkit_tree::format_tree(&entry, "", true, true, &mut lines);
            for line in lines {
                println!("{}", line);
            }
            println!("{}", devkit_tree::tree_summary(&entry));
        }
        Commands::Block { command } => {
            match command {
                BlockCommands::Outline { file, imports, docstrings } => {
                    let path = std::path::Path::new(file);
                    match devkit_block::outline_file(path, *imports, *docstrings) {
                        Ok(lines) => {
                            for line in lines {
                                println!("{}", line);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                BlockCommands::Context { file, function, margin } => {
                    let path = std::path::Path::new(file);
                    match devkit_block::extract_context(path, function, *margin) {
                        Ok(ctx) => print!("{}", ctx),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                BlockCommands::Extract { file, lines, marker, heading, function } => {
                    let path = std::path::Path::new(file);
                    match devkit_block::extract_block(
                        path,
                        lines.as_deref(),
                        marker.as_deref(),
                        heading.as_deref(),
                        function.as_deref(),
                    ) {
                        Ok(block) => print!("{}", block),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                BlockCommands::Replace { file, with_file, lines, marker, heading, function, dry_run } => {
                    let path = std::path::Path::new(file);
                    let with_path = std::path::Path::new(with_file);
                    let replacement = std::fs::read_to_string(with_path).unwrap_or_else(|e| {
                        eprintln!("Error reading replacement file: {}", e);
                        std::process::exit(1);
                    });
                    match devkit_block::replace_block(
                        path,
                        &replacement,
                        lines.as_deref(),
                        marker.as_deref(),
                        heading.as_deref(),
                        function.as_deref(),
                        *dry_run,
                    ) {
                        Ok((old, new_b)) => {
                            if *dry_run {
                                println!("DRY RUN: file was not modified.");
                                let diff = devkit_block::diff_preview(&old, &new_b, path);
                                print!("{}", diff);
                            } else {
                                println!("Successfully replaced block in {}", file);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Md { command } => {
            let get_content = |c: &Option<String>, w: &Option<String>| -> String {
                if let Some(text) = c {
                    text.clone()
                } else if let Some(path) = w {
                    std::fs::read_to_string(path).unwrap_or_else(|e| {
                        eprintln!("Error reading file {}: {}", path, e);
                        std::process::exit(1);
                    })
                } else {
                    String::new()
                }
            };
            
            match command {
                MdCommands::AppendSection { file, heading, content, with_file, dry_run } => {
                    let path = std::path::Path::new(file);
                    let body = get_content(content, with_file);
                    match devkit_md::append_to_section(path, heading, body, *dry_run) {
                        Ok(_) => println!("Successfully appended to section '{}'", heading),
                        Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
                    }
                }
                MdCommands::ReplaceSection { file, heading, content, with_file, no_keep_heading, dry_run } => {
                    let path = std::path::Path::new(file);
                    let body = get_content(content, with_file);
                    match devkit_md::replace_section(path, heading, body, !*no_keep_heading, *dry_run) {
                        Ok(_) => println!("Successfully replaced section '{}'", heading),
                        Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
                    }
                }
                MdCommands::EnsureSection { file, heading, content, with_file, level, after, dry_run } => {
                    let path = std::path::Path::new(file);
                    let body = get_content(content, with_file);
                    match devkit_md::ensure_section(path, heading, body, *level, after.as_deref(), *dry_run) {
                        Ok(_) => println!("Successfully ensured section '{}'", heading),
                        Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
                    }
                }
                MdCommands::AppendBullet { file, heading, bullet, dedupe, dry_run } => {
                    let path = std::path::Path::new(file);
                    match devkit_md::append_bullet(path, heading, bullet, *dedupe, *dry_run) {
                        Ok(_) => println!("Successfully appended bullet to '{}'", heading),
                        Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
                    }
                }
            }
        }
        Commands::Patch { command } => {
            match command {
                PatchCommands::Diagnose { file } => {
                    let path = std::path::Path::new(file);
                    let diag = devkit_patch::diagnose_patch(path);
                    println!("{}", diag.summary());
                    if !diag.success {
                        std::process::exit(1);
                    }
                }
                PatchCommands::Apply { file, reject, verbose } => {
                    let path = std::path::Path::new(file);
                    let diag = devkit_patch::apply_patch(path, false, *verbose, *reject);
                    println!("{}", diag.summary());
                    if !diag.success {
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Doc { command } => println!("doc {:?}: Not implemented yet", command),
        Commands::Git { command } => println!("git {:?}: Not implemented yet", command),
    }

    if cli.brief {
        println!("(brief mode enabled)");
    }
}
