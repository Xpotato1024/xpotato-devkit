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

    /// Diff operations
    Diff {
        #[command(subcommand)]
        command: DiffCommands,
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
    Diagnose { file: String },
    Apply {
        file: String,
        #[arg(long)]
        reject: bool,
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DiffCommands {
    Summarize {
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        head: Option<String>,
        #[arg(long)]
        commits: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        files_only: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DocCommands {
    ImplNote {
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        head: Option<String>,
        #[arg(long)]
        commits: Option<String>,
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        output: Option<String>,
    },
    BenchmarkNote {
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        head: Option<String>,
        #[arg(long)]
        commits: Option<String>,
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GitCommands {
    CommitMessage {
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        head: Option<String>,
        #[arg(long)]
        commits: Option<String>,
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        output: Option<String>,
    },
    PrBody {
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        head: Option<String>,
        #[arg(long)]
        commits: Option<String>,
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        output: Option<String>,
    },
    SafePush {
        #[arg(long, short = 'y')]
        yes: bool,
        #[arg(long)]
        no_confirm: bool,
        #[arg(long)]
        remote: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Tree {
            path,
            max_depth,
            ext,
            dirs_only,
            no_gitignore,
        } => {
            use std::path::PathBuf;
            let root = path
                .as_ref()
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
        Commands::Block { command } => match command {
            BlockCommands::Outline {
                file,
                imports,
                docstrings,
            } => {
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
            BlockCommands::Context {
                file,
                function,
                margin,
            } => {
                let path = std::path::Path::new(file);
                match devkit_block::extract_context(path, function, *margin) {
                    Ok(ctx) => print!("{}", ctx),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            BlockCommands::Extract {
                file,
                lines,
                marker,
                heading,
                function,
            } => {
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
            BlockCommands::Replace {
                file,
                with_file,
                lines,
                marker,
                heading,
                function,
                dry_run,
            } => {
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
        },
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
                MdCommands::AppendSection {
                    file,
                    heading,
                    content,
                    with_file,
                    dry_run,
                } => {
                    let path = std::path::Path::new(file);
                    let body = get_content(content, with_file);
                    match devkit_md::append_to_section(path, heading, body, *dry_run) {
                        Ok(_) => println!("Successfully appended to section '{}'", heading),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                MdCommands::ReplaceSection {
                    file,
                    heading,
                    content,
                    with_file,
                    no_keep_heading,
                    dry_run,
                } => {
                    let path = std::path::Path::new(file);
                    let body = get_content(content, with_file);
                    match devkit_md::replace_section(
                        path,
                        heading,
                        body,
                        !*no_keep_heading,
                        *dry_run,
                    ) {
                        Ok(_) => println!("Successfully replaced section '{}'", heading),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                MdCommands::EnsureSection {
                    file,
                    heading,
                    content,
                    with_file,
                    level,
                    after,
                    dry_run,
                } => {
                    let path = std::path::Path::new(file);
                    let body = get_content(content, with_file);
                    match devkit_md::ensure_section(
                        path,
                        heading,
                        body,
                        *level,
                        after.as_deref(),
                        *dry_run,
                    ) {
                        Ok(_) => println!("Successfully ensured section '{}'", heading),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                MdCommands::AppendBullet {
                    file,
                    heading,
                    bullet,
                    dedupe,
                    dry_run,
                } => {
                    let path = std::path::Path::new(file);
                    match devkit_md::append_bullet(path, heading, bullet, *dedupe, *dry_run) {
                        Ok(_) => println!("Successfully appended bullet to '{}'", heading),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Patch { command } => match command {
            PatchCommands::Diagnose { file } => {
                let path = std::path::Path::new(file);
                let diag = devkit_patch::diagnose_patch(path);
                println!("{}", diag.summary());
                if !diag.success {
                    std::process::exit(1);
                }
            }
            PatchCommands::Apply {
                file,
                reject,
                verbose,
            } => {
                let path = std::path::Path::new(file);
                let diag = devkit_patch::apply_patch(path, false, *verbose, *reject);
                println!("{}", diag.summary());
                if !diag.success {
                    std::process::exit(1);
                }
            }
        },
        Commands::Diff { command } => match command {
            DiffCommands::Summarize {
                staged,
                base,
                head,
                commits,
                json,
                files_only,
            } => {
                match devkit_git::diff::summarize_diff(
                    *staged,
                    base.as_deref(),
                    head.as_deref(),
                    commits.as_deref(),
                ) {
                    Ok(summary) => {
                        if *json {
                            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
                        } else if *files_only {
                            for f in &summary.files {
                                println!("{}", f.path);
                            }
                        } else if cli.brief {
                            println!(
                                "OK: {} files, +{}/-{}",
                                summary.files.len(),
                                summary.total_additions,
                                summary.total_deletions
                            );
                        } else {
                            println!("Diff Summary ({})", summary.scope.description);
                            println!("{:-<50}", "-");
                            println!("{:<40} {:>5} {:>5}", "File", "(+)", "(-)");
                            for f in &summary.files {
                                let adds = if f.is_binary {
                                    "bin".to_string()
                                } else {
                                    f.additions.to_string()
                                };
                                let dels = if f.is_binary {
                                    "bin".to_string()
                                } else {
                                    f.deletions.to_string()
                                };
                                println!("{:<40} {:>5} {:>5}", f.path, adds, dels);
                            }
                            println!("{:-<50}", "-");
                            println!(
                                "{:<40} {:>5} {:>5}",
                                "Total", summary.total_additions, summary.total_deletions
                            );
                        }
                    }
                    Err(e) => {
                        if cli.brief {
                            println!("FAIL: {}", e);
                        } else {
                            eprintln!("Error: {}", e);
                        }
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Doc { command } => {
            let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let config = devkit_core::load_config(&root).unwrap_or_default();

            match command {
                DocCommands::ImplNote {
                    staged,
                    base,
                    head,
                    commits,
                    lang,
                    output,
                } => {
                    let conf_lang = if config.git.lang.is_empty() {
                        "ja"
                    } else {
                        config.git.lang.as_str()
                    };
                    let lang_to_use = lang.as_deref().unwrap_or(conf_lang);
                    let summary = devkit_git::diff::summarize_diff(
                        *staged,
                        base.as_deref(),
                        head.as_deref(),
                        commits.as_deref(),
                    )
                    .ok();
                    let content =
                        devkit_git::doc::generate_impl_note(summary.as_ref(), lang_to_use);
                    if let Some(path) = output {
                        if std::fs::write(path, &content).is_ok() {
                            println!("Implementation note template written to {}", path);
                        } else {
                            eprintln!("Failed to write to {}", path);
                        }
                    } else {
                        print!("{}", content);
                    }
                }
                DocCommands::BenchmarkNote {
                    staged,
                    base,
                    head,
                    commits,
                    lang,
                    output,
                } => {
                    let conf_lang = if config.git.lang.is_empty() {
                        "ja"
                    } else {
                        config.git.lang.as_str()
                    };
                    let lang_to_use = lang.as_deref().unwrap_or(conf_lang);
                    let summary = devkit_git::diff::summarize_diff(
                        *staged,
                        base.as_deref(),
                        head.as_deref(),
                        commits.as_deref(),
                    )
                    .ok();
                    let content =
                        devkit_git::doc::generate_benchmark_note(summary.as_ref(), lang_to_use);
                    if let Some(path) = output {
                        if std::fs::write(path, &content).is_ok() {
                            println!("Benchmark note template written to {}", path);
                        } else {
                            eprintln!("Failed to write to {}", path);
                        }
                    } else {
                        print!("{}", content);
                    }
                }
            }
        }
        Commands::Git { command } => {
            let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let config = devkit_core::load_config(&root).unwrap_or_default();

            match command {
                GitCommands::CommitMessage {
                    staged,
                    base,
                    head,
                    commits,
                    lang,
                    output,
                } => {
                    let conf_lang = if config.git.lang.is_empty() {
                        "ja"
                    } else {
                        config.git.lang.as_str()
                    };
                    let lang_to_use = lang.as_deref().unwrap_or(conf_lang);
                    match devkit_git::git::generate_commit_template(
                        *staged,
                        base.as_deref(),
                        head.as_deref(),
                        commits.as_deref(),
                        lang_to_use,
                    ) {
                        Ok(content) => {
                            if let Some(path) = output {
                                if std::fs::write(path, &content).is_ok() {
                                    println!("Commit message template written to {}", path);
                                } else {
                                    eprintln!("Failed to write to {}", path);
                                }
                            } else {
                                print!("{}", content);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                GitCommands::PrBody {
                    staged,
                    base,
                    head,
                    commits,
                    lang,
                    output,
                } => {
                    let conf_lang = if config.git.lang.is_empty() {
                        "ja"
                    } else {
                        config.git.lang.as_str()
                    };
                    let lang_to_use = lang.as_deref().unwrap_or(conf_lang);
                    match devkit_git::git::generate_pr_template(
                        *staged,
                        base.as_deref(),
                        head.as_deref(),
                        commits.as_deref(),
                        lang_to_use,
                    ) {
                        Ok(content) => {
                            if let Some(path) = output {
                                if std::fs::write(path, &content).is_ok() {
                                    println!("PR body template written to {}", path);
                                } else {
                                    eprintln!("Failed to write to {}", path);
                                }
                            } else {
                                print!("{}", content);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                GitCommands::SafePush {
                    yes,
                    no_confirm,
                    remote,
                } => {
                    if let Err(e) = devkit_git::git::check_safe_branch() {
                        eprintln!("Safety Check Failed: {}", e);
                        std::process::exit(1);
                    }
                    let current = devkit_git::git::get_current_branch().unwrap_or_default();
                    println!("Pushing branch {}...", current);

                    let mut args = vec!["push"];
                    let mut tracking_set = false;
                    let target_remote;
                    if !devkit_git::git::check_upstream() {
                        if let Some(r) = remote.as_ref() {
                            println!("No upstream set. Will automatically set upstream to {}.", r);
                            args.push("-u");
                            args.push(r.as_str());
                            args.push(current.as_str());
                            tracking_set = true;
                            target_remote = r.clone();
                        } else {
                            eprintln!(
                                "No upstream is configured for this branch. Use --remote to set one."
                            );
                            std::process::exit(1);
                        }
                    } else {
                        target_remote = devkit_git::git::get_upstream_remote().unwrap_or_default();
                    }

                    let do_push = *yes || *no_confirm;
                    if !do_push {
                        println!(
                            "Dry run mode or interactive prompt not implemented in Rust safe-push yet."
                        );
                        println!("Use --yes to confirm.");
                        std::process::exit(1);
                    }

                    let status = std::process::Command::new("git").args(&args).status();
                    if let Ok(st) = status {
                        if !st.success() {
                            eprintln!("Push failed.");
                            std::process::exit(1);
                        } else {
                            let track_msg = if tracking_set { "set" } else { "unchanged" };
                            println!(
                                "Successfully pushed branch {} to {}. Tracking: {}.",
                                current, target_remote, track_msg
                            );
                        }
                    } else {
                        eprintln!("Push failed to execute.");
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    if cli.brief {
        println!("(brief mode enabled)");
    }
}
