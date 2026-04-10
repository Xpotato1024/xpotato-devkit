use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn next_temp_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "devkit-cli-smoke-{name}-{}-{unique}-{}",
            std::process::id(),
            next_temp_id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn devkit() -> &'static str {
    env!("CARGO_BIN_EXE_devkit")
}

fn run_devkit(cwd: &Path, args: &[&str]) -> Output {
    Command::new(devkit())
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

fn run_git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed:\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn init_repo() -> TempDir {
    let temp = TempDir::new("repo");
    run_git(&temp.path, &["init", "-q"]);
    run_git(&temp.path, &["config", "user.email", "devkit@example.com"]);
    run_git(&temp.path, &["config", "user.name", "devkit"]);

    write(
        &temp.path.join("README.md"),
        "# Sample\n\n## Usage\nInitial body\n",
    );
    write(
        &temp.path.join("src/lib.rs"),
        "pub struct Widget;\nimpl Widget {\n    pub fn helper() {}\n}\n",
    );
    run_git(&temp.path, &["add", "."]);
    run_git(&temp.path, &["commit", "-qm", "init"]);

    write(
        &temp.path.join("README.md"),
        "# Sample\n\n## Usage\nUpdated staged body\n",
    );
    run_git(&temp.path, &["add", "README.md"]);
    write(
        &temp.path.join("src/lib.rs"),
        "pub struct Widget;\nimpl Widget {\n    pub fn helper() {}\n    pub fn extra() {}\n}\n",
    );
    temp
}

#[test]
fn help_smoke_for_current_command_groups() {
    let commands = [
        &["--help"][..],
        &["tree", "--help"],
        &["encoding", "--help"],
        &["bootstrap", "--help"],
        &["block", "--help"],
        &["diff", "--help"],
        &["search", "--help"],
        &["patch", "--help"],
        &["doc", "--help"],
        &["git", "--help"],
        &["metrics", "--help"],
    ];

    for args in commands {
        let output = run_devkit(Path::new("."), args);
        assert!(
            output.status.success(),
            "help failed for {:?}: {}",
            args,
            stderr(&output)
        );
    }
}

#[test]
fn runtime_smoke_covers_search_patch_and_existing_commands() {
    let temp = init_repo();
    write(&temp.path.join("invalid.patch"), "not a patch\njust text\n");

    let cases = [
        (
            &["tree", "--path", ".", "--max-depth", "2", "--json"][..],
            0,
            "directories",
        ),
        (&["encoding", "check", "README.md", "--brief"], 0, "OK:"),
        (
            &["block", "extract", "README.md", "--list-headings"],
            0,
            "Usage",
        ),
        (
            &["block", "extract", "README.md", "--lines", "1-999"],
            0,
            "# Sample",
        ),
        (
            &["diff", "summarize", "--unstaged", "--json"],
            0,
            "\"scope\"",
        ),
        (
            &["search", "text", "extra", "--type", "rust", "--json"],
            0,
            "\"mode\": \"text\"",
        ),
        (
            &[
                "search",
                "symbol",
                "Widget|helper",
                "--type",
                "rust",
                "--json",
            ],
            0,
            "\"mode\": \"symbol\"",
        ),
        (
            &[
                "patch",
                "diagnose",
                "--patch-file",
                "invalid.patch",
                "--brief",
            ],
            1,
            "invalid patch input",
        ),
        (&["doc", "impl-note", "--staged"], 0, "##"),
        (&["git", "commit-message", "--staged"], 0, "README.md"),
        (
            &["metrics", "show", "--path", ".devkit-metrics.jsonl"],
            0,
            "does not exist yet",
        ),
    ];

    for (args, code, needle) in cases {
        let output = run_devkit(&temp.path, args);
        assert_eq!(
            output.status.code(),
            Some(code),
            "unexpected exit code for {:?}\nstdout={}\nstderr={}",
            args,
            stdout(&output),
            stderr(&output)
        );
        let combined = format!("{}\n{}", stdout(&output), stderr(&output));
        assert!(
            combined.contains(needle),
            "missing {:?} in output for {:?}\n{}",
            needle,
            args,
            combined
        );
    }
}

#[test]
fn search_usage_errors_exit_with_code_2() {
    let temp = init_repo();
    let output = run_devkit(
        &temp.path,
        &[
            "search",
            "text",
            "helper",
            "--files-with-matches",
            "--count",
        ],
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("cannot be combined"));
}

#[test]
fn bootstrap_runtime_can_sync_skills_and_init_agents() {
    let temp = TempDir::new("bootstrap");
    let repo_root = temp.path.join("repo");
    let workspace = temp.path.join("workspace");
    write(
        &repo_root.join("SKILLs").join("demo-skill").join("SKILL.md"),
        "# demo-skill\n",
    );

    let sync = run_devkit(
        &temp.path,
        &[
            "bootstrap",
            "sync-skills",
            "--repo-root",
            repo_root.to_str().unwrap(),
            "--target",
            workspace.to_str().unwrap(),
        ],
    );
    assert_eq!(sync.status.code(), Some(0), "{}", stderr(&sync));
    assert!(
        workspace
            .join("SKILLs")
            .join("demo-skill")
            .join("SKILL.md")
            .is_file(),
        "sync-skills did not copy the skill tree"
    );

    let agents_path = workspace.join("AGENTS.md");
    let init_agents = run_devkit(
        &temp.path,
        &[
            "bootstrap",
            "init-agents",
            "--path",
            agents_path.to_str().unwrap(),
        ],
    );
    assert_eq!(
        init_agents.status.code(),
        Some(0),
        "{}",
        stderr(&init_agents)
    );
    let agents = fs::read_to_string(&agents_path).unwrap();
    assert!(agents.contains("demo-skill"));
    assert!(agents.contains("SKILLs/"));
}
