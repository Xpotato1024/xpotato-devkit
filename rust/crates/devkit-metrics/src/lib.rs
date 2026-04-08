use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRecord {
    pub ts: String,
    pub cmd: String,
    pub duration_ms: f64,
    pub brief: bool,
    pub ok: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricSummary {
    pub count: usize,
    pub total_ms: f64,
    pub avg_ms: f64,
    pub success_count: usize,
    pub success_rate: f64,
    pub brief_count: usize,
}

pub fn get_metrics_file(cwd: &Path) -> Option<PathBuf> {
    let base = devkit_core::config_base_dir(cwd);
    let config = devkit_core::load_config(cwd).ok()?;
    if !config.metrics.enabled {
        return None;
    }

    let relative = if config.metrics.path.is_empty() {
        ".devkit-metrics.jsonl"
    } else {
        config.metrics.path.as_str()
    };

    Some(base.join(relative))
}

pub fn record_metric(cwd: &Path, cmd: &str, duration_ms: f64, brief: bool, ok: bool) {
    let Some(target) = get_metrics_file(cwd) else {
        return;
    };

    let parent = target.parent().map(Path::to_path_buf);
    if let Some(dir) = parent {
        let _ = fs::create_dir_all(dir);
    }

    let record = MetricRecord {
        ts: chrono_like_timestamp(),
        cmd: cmd.to_string(),
        duration_ms,
        brief,
        ok,
    };

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&target) {
        let _ = writeln!(file, "{}", serde_json::to_string(&record).unwrap());
    }
}

pub fn load_metrics(filepath: &Path) -> Vec<MetricRecord> {
    let Ok(content) = fs::read_to_string(filepath) else {
        return Vec::new();
    };

    content
        .lines()
        .filter_map(|line| serde_json::from_str::<MetricRecord>(line).ok())
        .collect()
}

pub fn summarize_metrics(records: &[MetricRecord]) -> BTreeMap<String, MetricSummary> {
    let mut summary = BTreeMap::new();

    for record in records {
        let entry = summary.entry(record.cmd.clone()).or_insert(MetricSummary {
            count: 0,
            total_ms: 0.0,
            avg_ms: 0.0,
            success_count: 0,
            success_rate: 0.0,
            brief_count: 0,
        });

        entry.count += 1;
        entry.total_ms += record.duration_ms;
        if record.ok {
            entry.success_count += 1;
        }
        if record.brief {
            entry.brief_count += 1;
        }
    }

    for entry in summary.values_mut() {
        if entry.count > 0 {
            entry.avg_ms = entry.total_ms / entry.count as f64;
            entry.success_rate = entry.success_count as f64 / entry.count as f64;
        }
    }

    summary
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("devkit-metrics-test-{}", unique));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn summarizes_records_by_command() {
        let records = vec![
            MetricRecord {
                ts: "1Z".to_string(),
                cmd: "block outline".to_string(),
                duration_ms: 100.0,
                brief: false,
                ok: true,
            },
            MetricRecord {
                ts: "2Z".to_string(),
                cmd: "block outline".to_string(),
                duration_ms: 200.0,
                brief: true,
                ok: true,
            },
            MetricRecord {
                ts: "3Z".to_string(),
                cmd: "patch apply".to_string(),
                duration_ms: 500.0,
                brief: false,
                ok: false,
            },
        ];

        let summary = summarize_metrics(&records);
        let outline = summary.get("block outline").unwrap();
        assert_eq!(outline.count, 2);
        assert_eq!(outline.total_ms, 300.0);
        assert_eq!(outline.avg_ms, 150.0);
        assert_eq!(outline.brief_count, 1);
        assert_eq!(outline.success_count, 2);
    }

    #[test]
    fn uses_configured_metrics_path() {
        let temp = TempDir::new();
        fs::write(
            temp.path.join("devkit.toml"),
            "[metrics]\nenabled = true\npath = \"logs/metrics.jsonl\"\n",
        )
        .unwrap();

        let target = get_metrics_file(&temp.path).unwrap();
        let expected = temp.path.join("logs").join("metrics.jsonl");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        assert_eq!(
            target.parent().unwrap().canonicalize().unwrap(),
            expected.parent().unwrap().canonicalize().unwrap()
        );
        assert_eq!(target.file_name(), expected.file_name());
    }

    #[test]
    fn uses_devkit_config_env_base_directory() {
        let _guard = env_lock().lock().unwrap();
        let temp = TempDir::new();
        let workspace = temp.path.join("workspace");
        let config_dir = temp.path.join("config-dir");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("custom.toml");
        fs::write(
            &config_path,
            "[metrics]\nenabled = true\npath = \"logs/metrics.jsonl\"\n",
        )
        .unwrap();

        unsafe { std::env::set_var("DEVKIT_CONFIG", &config_path) };
        let target = get_metrics_file(&workspace).unwrap();
        unsafe { std::env::remove_var("DEVKIT_CONFIG") };

        assert_eq!(
            target
                .to_string_lossy()
                .replace("\\\\?\\", "")
                .replace('/', "\\"),
            config_dir
                .join("logs")
                .join("metrics.jsonl")
                .to_string_lossy()
                .replace("\\\\?\\", "")
                .replace('/', "\\")
        );
    }
}
