use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

pub struct FileWatcher {
    debouncer: Option<Debouncer<notify::RecommendedWatcher>>,
    rx: Receiver<PathBuf>,
    tx: Sender<PathBuf>,
    current_path: Option<PathBuf>,
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl FileWatcher {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            debouncer: None,
            rx,
            tx,
            current_path: None,
        }
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), String> {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

        if let Some(ref current) = self.current_path {
            if current == &canonical {
                return Ok(());
            }
            if let Some(ref mut debouncer) = self.debouncer {
                let _ = debouncer.watcher().unwatch(current);
            }
        }

        let tx = self.tx.clone();
        let watched_file = canonical.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(250),
            move |res: Result<Vec<DebouncedEvent>, _>| {
                if let Ok(events) = res {
                    for event in events {
                        if event.path == watched_file {
                            let _ = tx.send(event.path);
                            break;
                        }
                    }
                }
            },
        )
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        debouncer
            .watcher()
            .watch(&canonical, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch file {}: {}", canonical.display(), e))?;

        self.debouncer = Some(debouncer);
        self.current_path = Some(canonical);

        Ok(())
    }

    pub fn unwatch(&mut self) {
        if let (Some(ref mut debouncer), Some(ref path)) = (&mut self.debouncer, &self.current_path) {
            let _ = debouncer.watcher().unwatch(path);
        }
        self.current_path = None;
        self.debouncer = None;
    }

    pub fn has_changes(&self) -> bool {
        let mut changed = false;
        while self.rx.try_recv().is_ok() {
            changed = true;
        }
        changed
    }

    pub fn is_watching(&self) -> bool {
        self.current_path.is_some()
    }

    pub fn watched_path(&self) -> Option<&Path> {
        self.current_path.as_deref()
    }
}
