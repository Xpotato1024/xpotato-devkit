use clap::{Parser, Subcommand, ValueEnum};
use std::time::Instant;

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

    /// Encoding and text sanity checks
    Encoding {
        #[command(subcommand)]
        command: EncodingCommands,
    },

    /// Install the current checkout as a local user tool
    Bootstrap {
        #[command(subcommand)]
        command: BootstrapCommands,
    },

    /// Manage devkit project configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
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

    /// Manage local devkit metrics
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum EncodingCommands {
    /// Check files for UTF-8 and newline anomalies
    Check { files: Vec<String> },
    /// Normalize BOM and newline style
    Normalize {
        files: Vec<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, value_enum, default_value_t = CliNewlineStyle::Lf)]
        newline: CliNewlineStyle,
    },
}

#[derive(Subcommand, Debug)]
pub enum BootstrapCommands {
    /// Install the current checkout via cargo
    InstallSelf {
        #[arg(long)]
        repo_root: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Write a starter devkit.toml template
    Init {
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum MetricsCommands {
    /// Show aggregated local usage metrics
    Show {
        #[arg(long)]
        path: Option<String>,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliNewlineStyle {
    Lf,
    Crlf,
}

impl From<CliNewlineStyle> for devkit_encoding::NewlineStyle {
    fn from(value: CliNewlineStyle) -> Self {
        match value {
            CliNewlineStyle::Lf => Self::Lf,
            CliNewlineStyle::Crlf => Self::Crlf,
        }
    }
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
        heading_exact: bool,
        #[arg(long)]
        function: Option<String>,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        list_headings: bool,
        #[arg(long)]
        list_functions: bool,
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
        heading_exact: bool,
        #[arg(long)]
        function: Option<String>,
        #[arg(long)]
        symbol: Option<String>,
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
        #[arg()]
        file: Option<String>,
        #[arg(long)]
        patch_file: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Apply {
        #[arg()]
        file: Option<String>,
        #[arg(long)]
        patch_file: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        reject: bool,
        #[arg(long)]
        verbose: bool,
        #[arg(long)]
        json: bool,
    },
}

fn render_patch_diagnostic(
    diag: &devkit_patch::PatchDiagnostic,
    command: &str,
    json: bool,
    brief: bool,
) -> Result<(), String> {
    if json && brief {
        return Err("`--json` and `--brief` cannot be combined for patch commands.".to_string());
    }

    if json {
        let payload = serde_json::json!({
            "command": command,
            "success": diag.success,
            "summary": if diag.success {
                diag.brief_summary()
            } else {
                diag.summary()
            },
            "total_hunks": diag.total_hunks,
            "applied_hunks": diag.applied_hunks,
            "failed_hunks": diag.failed_hunks,
            "errors": &diag.errors,
            "affected_files": &diag.affected_files,
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        return Ok(());
    }

    if brief {
        println!("{}", diag.brief_summary());
    } else {
        println!("{}", diag.summary());
    }

    Ok(())
}

fn format_encoding_result_line(result: &devkit_encoding::EncodingCheckResult) -> String {
    if result.has_issues() {
        let mut issues = result.issue_labels();
        if result.error.is_some() && !issues.contains(&"error") {
            issues.push("error");
        }
        format!("FAIL\t{}\t{}", result.file, issues.join(", "))
    } else {
        format!("OK\t{}\tclean", result.file)
    }
}

fn format_encoding_brief(results: &[devkit_encoding::EncodingCheckResult]) -> String {
    let issue_details: Vec<String> = results
        .iter()
        .filter(|result| result.has_issues())
        .map(|result| format!("{}: {}", result.file, result.issue_labels().join(", ")))
        .collect();

    if issue_details.is_empty() {
        format!("OK: {} files checked, 0 issues", results.len())
    } else {
        let detail = issue_details
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("; ");
        let extra = if issue_details.len() > 5 {
            format!(" +{} more", issue_details.len() - 5)
        } else {
            String::new()
        };
        format!(
            "FAIL: {} files checked, {} issues ({}{})",
            results.len(),
            issue_details.len(),
            detail,
            extra
        )
    }
}

fn format_normalize_brief(results: &[devkit_encoding::NormalizeResult], dry_run: bool) -> String {
    let changed = results.iter().filter(|result| result.changed).count();
    if dry_run {
        format!(
            "OK: {} files checked, {} files would change (dry-run)",
            results.len(),
            changed
        )
    } else {
        format!(
            "OK: {} files checked, {} files changed",
            results.len(),
            changed
        )
    }
}

fn command_label(command: &Commands) -> &'static str {
    match command {
        Commands::Tree { .. } => "tree",
        Commands::Encoding { command } => match command {
            EncodingCommands::Check { .. } => "encoding check",
            EncodingCommands::Normalize { .. } => "encoding normalize",
        },
        Commands::Bootstrap { command } => match command {
            BootstrapCommands::InstallSelf { .. } => "bootstrap install-self",
        },
        Commands::Config { command } => match command {
            ConfigCommands::Init { .. } => "config init",
        },
        Commands::Block { command } => match command {
            BlockCommands::Outline { .. } => "block outline",
            BlockCommands::Context { .. } => "block context",
            BlockCommands::Extract { .. } => "block extract",
            BlockCommands::Replace { .. } => "block replace",
        },
        Commands::Md { command } => match command {
            MdCommands::AppendSection { .. } => "md append-section",
            MdCommands::ReplaceSection { .. } => "md replace-section",
            MdCommands::EnsureSection { .. } => "md ensure-section",
            MdCommands::AppendBullet { .. } => "md append-bullet",
        },
        Commands::Patch { command } => match command {
            PatchCommands::Diagnose { .. } => "patch diagnose",
            PatchCommands::Apply { .. } => "patch apply",
        },
        Commands::Diff { command } => match command {
            DiffCommands::Summarize { .. } => "diff summarize",
        },
        Commands::Doc { command } => match command {
            DocCommands::ImplNote { .. } => "doc impl-note",
            DocCommands::BenchmarkNote { .. } => "doc benchmark-note",
        },
        Commands::Git { command } => match command {
            GitCommands::CommitMessage { .. } => "git commit-message",
            GitCommands::PrBody { .. } => "git pr-body",
            GitCommands::SafePush { .. } => "git safe-push",
        },
        Commands::Metrics { command } => match command {
            MetricsCommands::Show { .. } => "metrics show",
        },
    }
}

fn record_metrics(cli: &Cli, start: Instant, ok: bool) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    devkit_metrics::record_metric(
        &cwd,
        command_label(&cli.command),
        duration_ms,
        cli.brief,
        ok,
    );
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
    let start = Instant::now();

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
                exit_with_timing(&cli, start, 1);
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
        Commands::Encoding { command } => match command {
            EncodingCommands::Check { files } => {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let config = match devkit_core::load_config(&cwd) {
                    Ok(config) => config,
                    Err(error) => {
                        if cli.brief {
                            println!("FAIL: {}", error);
                        } else {
                            eprintln!("Error: {}", error);
                        }
                        exit_with_timing(&cli, start, 1);
                    }
                };
                let ignore_patterns =
                    devkit_encoding::resolve_ignore_patterns(&config.encoding.ignore);

                let inputs = match devkit_encoding::collect_inputs(&cwd, files) {
                    Ok(inputs) => inputs,
                    Err(error) => {
                        if cli.brief {
                            println!("FAIL: {}", error);
                        } else {
                            eprintln!("Error: {}", error);
                        }
                        exit_with_timing(&cli, start, 1);
                    }
                };

                let results: Vec<_> = inputs
                    .into_iter()
                    .filter(|path| path.is_file())
                    .filter(|path| !devkit_encoding::should_ignore(path, &ignore_patterns))
                    .map(|path| {
                        let display_path = devkit_encoding::display_path(&cwd, &path);
                        devkit_encoding::check_encoding(&path, display_path)
                    })
                    .collect();

                if results.is_empty() {
                    if cli.brief {
                        println!("FAIL: no valid files found");
                    } else {
                        eprintln!("Error: No valid files found to process.");
                    }
                    exit_with_timing(&cli, start, 1);
                }

                if cli.brief {
                    println!("{}", format_encoding_brief(&results));
                } else {
                    for result in &results {
                        println!("{}", format_encoding_result_line(result));
                    }
                    let issue_count = results.iter().filter(|result| result.has_issues()).count();
                    println!(
                        "Summary\t{} files checked\t{} issues",
                        results.len(),
                        issue_count
                    );
                }

                if results.iter().any(|result| result.has_issues()) {
                    exit_with_timing(&cli, start, 1);
                }
            }
            EncodingCommands::Normalize {
                files,
                dry_run,
                newline,
            } => {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let config = match devkit_core::load_config(&cwd) {
                    Ok(config) => config,
                    Err(error) => {
                        if cli.brief {
                            println!("FAIL: {}", error);
                        } else {
                            eprintln!("Error: {}", error);
                        }
                        exit_with_timing(&cli, start, 1);
                    }
                };
                let ignore_patterns =
                    devkit_encoding::resolve_ignore_patterns(&config.encoding.ignore);
                let inputs = match devkit_encoding::collect_inputs(&cwd, files) {
                    Ok(inputs) => inputs,
                    Err(error) => {
                        if cli.brief {
                            println!("FAIL: {}", error);
                        } else {
                            eprintln!("Error: {}", error);
                        }
                        exit_with_timing(&cli, start, 1);
                    }
                };

                let mut results = Vec::new();
                for path in inputs
                    .into_iter()
                    .filter(|path| path.is_file())
                    .filter(|path| !devkit_encoding::should_ignore(path, &ignore_patterns))
                {
                    let display_path = devkit_encoding::display_path(&cwd, &path);
                    match devkit_encoding::normalize_encoding(
                        &path,
                        display_path,
                        (*newline).into(),
                        *dry_run,
                    ) {
                        Ok(result) => results.push(result),
                        Err(error) => {
                            if cli.brief {
                                println!("FAIL: {}", error);
                            } else {
                                eprintln!("Error: {}", error);
                            }
                            exit_with_timing(&cli, start, 1);
                        }
                    }
                }

                if results.is_empty() {
                    if cli.brief {
                        println!("FAIL: no valid files found");
                    } else {
                        eprintln!("Error: No valid files found to process.");
                    }
                    exit_with_timing(&cli, start, 1);
                }

                if cli.brief {
                    println!("{}", format_normalize_brief(&results, *dry_run));
                } else {
                    for result in &results {
                        let status = if result.changed {
                            if *dry_run { "WOULD_CHANGE" } else { "CHANGED" }
                        } else {
                            "UNCHANGED"
                        };
                        println!("{}\t{}", status, result.file);
                    }
                }
            }
        },
        Commands::Bootstrap { command } => match command {
            BootstrapCommands::InstallSelf { repo_root } => {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let resolved_root = if let Some(path) = repo_root {
                    std::path::PathBuf::from(path)
                } else {
                    match devkit_bootstrap::find_repo_root(&cwd) {
                        Ok(path) => path,
                        Err(error) => {
                            eprintln!("Error: {}", error);
                            exit_with_timing(&cli, start, 1);
                        }
                    }
                };

                match devkit_bootstrap::bootstrap_self(&resolved_root) {
                    Ok(tool_bin) => {
                        println!(
                            "Bootstrap complete. If the current shell does not see devkit yet, restart it or add {} to PATH.",
                            tool_bin.display()
                        );
                    }
                    Err(error) => {
                        eprintln!("Error: {}", error);
                        exit_with_timing(&cli, start, 1);
                    }
                }
            }
        },
        Commands::Config { command } => match command {
            ConfigCommands::Init { path, force } => {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let target = path
                    .as_ref()
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| cwd.join("devkit.toml"));

                if target.exists() && !force {
                    eprintln!(
                        "Error: {} already exists. Use --force to overwrite.",
                        target.display()
                    );
                    exit_with_timing(&cli, start, 1);
                }

                let template = r#"[encoding]
# Ignore files and directories during encoding/tree operations.
ignore = [".git", ".venv", "venv", "node_modules", "__pycache__", ".ruff_cache", "dist", "build"]

[git]
# Prompt language for generated commit / PR text.
lang = "ja"

[metrics]
# Enable local JSONL metrics collection when desired.
enabled = false
path = ".devkit-metrics.jsonl"
"#;

                if let Some(parent) = target.parent()
                    && let Err(error) = std::fs::create_dir_all(parent)
                {
                    eprintln!("Error: {}", error);
                    exit_with_timing(&cli, start, 1);
                }

                if let Err(error) = std::fs::write(&target, template) {
                    eprintln!("Error: {}", error);
                    exit_with_timing(&cli, start, 1);
                }

                println!("Wrote {}", target.display());
            }
        },
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
                        exit_with_timing(&cli, start, 1);
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
                        exit_with_timing(&cli, start, 1);
                    }
                }
            }
            BlockCommands::Extract {
                file,
                lines,
                marker,
                heading,
                function,
                symbol,
                list_headings,
                list_functions,
                heading_exact,
            } => {
                let path = std::path::Path::new(file);
                if symbol.is_some() && function.is_some() {
                    eprintln!("Error: Use either --function or --symbol, not both.");
                    exit_with_timing(&cli, start, 1);
                }
                if *list_headings && *list_functions {
                    eprintln!("Error: Use either --list-headings or --list-functions, not both.");
                    exit_with_timing(&cli, start, 1);
                }
                if (*list_headings || *list_functions)
                    && (lines.is_some()
                        || marker.is_some()
                        || heading.is_some()
                        || function.is_some()
                        || symbol.is_some())
                {
                    eprintln!("Error: Listing options cannot be combined with extract selectors.");
                    exit_with_timing(&cli, start, 1);
                }

                if *list_headings {
                    match devkit_block::list_markdown_headings(path) {
                        Ok(entries) => {
                            if entries.is_empty() {
                                println!("No entries found.");
                            } else {
                                for entry in entries {
                                    println!(
                                        "L{}: {} {}  [{}]",
                                        entry.line,
                                        "#".repeat(entry.level),
                                        entry.text,
                                        entry.slug
                                    );
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("Error: {}", error);
                            exit_with_timing(&cli, start, 1);
                        }
                    }
                    continue_or_return_after_branch(&cli, start);
                }

                if *list_functions {
                    match devkit_block::list_functions(path) {
                        Ok(entries) => {
                            if entries.is_empty() {
                                println!("No entries found.");
                            } else {
                                for entry in entries {
                                    println!("L{}: {}", entry.line, entry.name);
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("Error: {}", error);
                            exit_with_timing(&cli, start, 1);
                        }
                    }
                    continue_or_return_after_branch(&cli, start);
                }

                let effective_function = symbol.as_deref().or(function.as_deref());
                match devkit_block::extract_block(
                    path,
                    lines.as_deref(),
                    marker.as_deref(),
                    heading.as_deref(),
                    effective_function,
                    *heading_exact,
                ) {
                    Ok(block) => print!("{}", block),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        exit_with_timing(&cli, start, 1);
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
                symbol,
                dry_run,
                heading_exact,
            } => {
                let path = std::path::Path::new(file);
                let with_path = std::path::Path::new(with_file);
                if symbol.is_some() && function.is_some() {
                    eprintln!("Error: Use either --function or --symbol, not both.");
                    exit_with_timing(&cli, start, 1);
                }
                let replacement = std::fs::read_to_string(with_path).unwrap_or_else(|e| {
                    eprintln!("Error reading replacement file: {}", e);
                    exit_with_timing(&cli, start, 1);
                });
                let effective_function = symbol.as_deref().or(function.as_deref());
                match devkit_block::replace_block(
                    path,
                    &replacement,
                    devkit_block::BlockOptions {
                        line_range: lines.as_deref(),
                        marker: marker.as_deref(),
                        heading: heading.as_deref(),
                        function: effective_function,
                        heading_exact: *heading_exact,
                    },
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
                        exit_with_timing(&cli, start, 1);
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
                        exit_with_timing(&cli, start, 1);
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
                            exit_with_timing(&cli, start, 1);
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
                            exit_with_timing(&cli, start, 1);
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
                            exit_with_timing(&cli, start, 1);
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
                            exit_with_timing(&cli, start, 1);
                        }
                    }
                }
            }
        }
        Commands::Patch { command } => match command {
            PatchCommands::Diagnose {
                file,
                patch_file,
                json,
            } => {
                let selected = patch_file.as_ref().or(file.as_ref());
                let Some(selected) = selected else {
                    eprintln!("Error: Provide a patch file via positional path or --patch-file.");
                    exit_with_timing(&cli, start, 2);
                };
                let path = std::path::Path::new(selected);
                let diag = devkit_patch::diagnose_patch(path);
                if let Err(e) = render_patch_diagnostic(&diag, "diagnose", *json, cli.brief) {
                    eprintln!("Error: {}", e);
                    exit_with_timing(&cli, start, 2);
                }
                if !diag.success {
                    exit_with_timing(&cli, start, 1);
                }
            }
            PatchCommands::Apply {
                file,
                patch_file,
                dry_run,
                reject,
                verbose,
                json,
            } => {
                let selected = patch_file.as_ref().or(file.as_ref());
                let Some(selected) = selected else {
                    eprintln!("Error: Provide a patch file via positional path or --patch-file.");
                    exit_with_timing(&cli, start, 2);
                };
                let path = std::path::Path::new(selected);
                let diag = devkit_patch::apply_patch(path, *dry_run, *verbose, *reject);
                if let Err(e) = render_patch_diagnostic(&diag, "apply", *json, cli.brief) {
                    eprintln!("Error: {}", e);
                    exit_with_timing(&cli, start, 2);
                }
                if !diag.success {
                    exit_with_timing(&cli, start, 1);
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
                if *json && cli.brief {
                    eprintln!("Error: `--json` and `--brief` cannot be combined.");
                    exit_with_timing(&cli, start, 2);
                }
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
                        exit_with_timing(&cli, start, 1);
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
                            exit_with_timing(&cli, start, 1);
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
                            exit_with_timing(&cli, start, 1);
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
                        exit_with_timing(&cli, start, 1);
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
                            exit_with_timing(&cli, start, 1);
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
                        exit_with_timing(&cli, start, 1);
                    }

                    let status = std::process::Command::new("git").args(&args).status();
                    if let Ok(st) = status {
                        if !st.success() {
                            eprintln!("Push failed.");
                            exit_with_timing(&cli, start, 1);
                        } else {
                            let track_msg = if tracking_set { "set" } else { "unchanged" };
                            println!(
                                "Successfully pushed branch {} to {}. Tracking: {}.",
                                current, target_remote, track_msg
                            );
                        }
                    } else {
                        eprintln!("Push failed to execute.");
                        exit_with_timing(&cli, start, 1);
                    }
                }
            }
        }
        Commands::Metrics { command } => match command {
            MetricsCommands::Show { path } => {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let target = path
                    .as_ref()
                    .map(std::path::PathBuf::from)
                    .or_else(|| devkit_metrics::get_metrics_file(&cwd));

                let Some(target) = target else {
                    println!("Metrics are not enabled in devkit.toml or no path configured.");
                    emit_timing(&cli, start);
                    record_metrics(&cli, start, true);
                    return;
                };

                if !target.is_file() {
                    println!("Metrics file {} does not exist yet.", target.display());
                    emit_timing(&cli, start);
                    record_metrics(&cli, start, true);
                    return;
                }

                let records = devkit_metrics::load_metrics(&target);
                if records.is_empty() {
                    println!("No valid records found in {}.", target.display());
                    emit_timing(&cli, start);
                    record_metrics(&cli, start, true);
                    return;
                }

                let summary = devkit_metrics::summarize_metrics(&records);
                println!("Devkit Usage Metrics (Total: {} runs)", records.len());
                println!("Command\tCount\tAvg Time (ms)\tBrief %\tSuccess %");
                for (cmd, st) in summary.iter().rev() {
                    let brief_pct = (st.brief_count as f64 / st.count as f64) * 100.0;
                    let success_pct = st.success_rate * 100.0;
                    println!(
                        "{}\t{}\t{:.1}\t{:.1}%\t{:.1}%",
                        cmd, st.count, st.avg_ms, brief_pct, success_pct
                    );
                }
            }
        },
    }

    emit_timing(&cli, start);
    record_metrics(&cli, start, true);
}

fn exit_with_timing(cli: &Cli, start: Instant, code: i32) -> ! {
    emit_timing(cli, start);
    record_metrics(cli, start, code == 0);
    std::process::exit(code);
}

fn emit_timing(cli: &Cli, start: Instant) {
    if !cli.time && !cli.time_json {
        return;
    }

    let total_ms = start.elapsed().as_secs_f64() * 1000.0;
    if let Some(output) = format_timing_output(cli, total_ms) {
        eprintln!("{}", output);
    }
}

fn format_timing_output(cli: &Cli, total_ms: f64) -> Option<String> {
    if cli.time_json {
        Some(
            serde_json::json!({
                "total_ms": total_ms
            })
            .to_string(),
        )
    } else if cli.time {
        Some(format!("[time] {:.3}ms", total_ms))
    } else {
        None
    }
}

fn continue_or_return_after_branch(cli: &Cli, start: Instant) -> ! {
    emit_timing(cli, start);
    record_metrics(cli, start, true);
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use devkit_encoding::EncodingCheckResult;

    fn test_cli() -> Cli {
        Cli {
            brief: false,
            time: false,
            time_json: false,
            command: Commands::Tree {
                path: None,
                max_depth: None,
                ext: None,
                dirs_only: false,
                no_gitignore: false,
            },
        }
    }

    #[test]
    fn formats_human_timing_output() {
        let mut cli = test_cli();
        cli.time = true;
        let output = format_timing_output(&cli, 12.3456).unwrap();
        assert_eq!(output, "[time] 12.346ms");
    }

    #[test]
    fn formats_json_timing_output() {
        let mut cli = test_cli();
        cli.time_json = true;
        let output = format_timing_output(&cli, 12.5).unwrap();
        assert_eq!(output, r#"{"total_ms":12.5}"#);
    }

    #[test]
    fn encoding_brief_ok_is_single_line() {
        let results = vec![EncodingCheckResult {
            file: "README.md".to_string(),
            valid_utf8: true,
            has_bom: false,
            has_replacement_char: false,
            has_control_chars: false,
            mixed_newlines: false,
            error: None,
        }];

        assert_eq!(
            format_encoding_brief(&results),
            "OK: 1 files checked, 0 issues"
        );
    }

    #[test]
    fn encoding_brief_fail_lists_first_issue() {
        let results = vec![EncodingCheckResult {
            file: "bad.txt".to_string(),
            valid_utf8: true,
            has_bom: true,
            has_replacement_char: false,
            has_control_chars: false,
            mixed_newlines: false,
            error: None,
        }];

        assert_eq!(
            format_encoding_brief(&results),
            "FAIL: 1 files checked, 1 issues (bad.txt: BOM)"
        );
    }
}
