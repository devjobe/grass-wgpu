use crossbeam_channel::{Receiver, TryRecvError};
use notify::{Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::path::PathBuf;

pub struct FileWatcher {
    pub watcher: RecommendedWatcher,
    pub receiver: Receiver<Result<Event>>,
    pub root: PathBuf,
}

impl Default for FileWatcher {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| {
            sender.send(res).expect("Watch event send failure.");
        })
        .expect("Failed to create filesystem watcher.");

        let root = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(manifest_dir)
        } else {
            std::env::current_exe()
                .map(|path| {
                    path.parent()
                        .map(|exe_parent_path| exe_parent_path.to_owned())
                        .unwrap()
                })
                .unwrap()
        };

        let watch_path = root.join("assets");

        watcher
            .watch(watch_path, RecursiveMode::Recursive)
            .expect("Failed to watch assets folder.");

        FileWatcher {
            watcher,
            receiver,
            root,
        }
    }
}

impl FileWatcher {
    pub fn collect_modified(&self) -> Option<std::collections::HashSet<PathBuf>> {
        let mut set = None;
        loop {
            let event = match self.receiver.try_recv() {
                Ok(res) => res.unwrap(),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("Filesystem watcher disconnected."),
            };

            if let notify::event::Event {
                kind: notify::event::EventKind::Modify(_),
                paths,
                ..
            } = event
            {
                set.get_or_insert(std::collections::HashSet::new())
                    .extend(paths);
            }
        }
        set
    }
}
