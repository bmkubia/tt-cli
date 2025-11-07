use crossterm::{cursor, execute};
use std::io::{stdout, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::sleep;

const BRAILLE_FRAMES: &[char] = &['⡿', '⣟', '⣯', '⣷', '⣾', '⣽', '⣻', '⢿'];
const FRAME_DELAY: Duration = Duration::from_millis(90);

pub struct ShimmerLoader {
    text: String,
}

pub struct LoaderHandle {
    notify: Arc<Notify>,
    join_handle: Option<JoinHandle<()>>,
}

impl ShimmerLoader {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn spawn(self) -> LoaderHandle {
        let notify = Arc::new(Notify::new());
        let notify_clone = Arc::clone(&notify);
        let text = self.text;

        let join_handle = tokio::spawn(async move {
            run_loader(text, notify_clone).await;
        });

        LoaderHandle {
            notify,
            join_handle: Some(join_handle),
        }
    }
}

impl LoaderHandle {
    pub async fn stop(&mut self) {
        self.notify.notify_waiters();

        if let Some(handle) = self.join_handle.take() {
            let _ = handle.await;
        }
    }
}

impl Drop for LoaderHandle {
    fn drop(&mut self) {
        self.notify.notify_waiters();

        if let Some(handle) = self.join_handle.take() {
            handle.abort();
        }
    }
}

async fn run_loader(text: String, notify: Arc<Notify>) {
    if text.is_empty() {
        return;
    }

    let _ = execute!(stdout(), cursor::Hide);
    let start = Instant::now();
    let mut frame_index = 0;

    loop {
        let spinner = BRAILLE_FRAMES[frame_index];
        let elapsed = format_elapsed(start.elapsed());
        let display = format!("\r{spinner} {} ({elapsed})   ", text);
        print!("{display}");
        let _ = stdout().flush();

        tokio::select! {
            _ = notify.notified() => break,
            _ = sleep(FRAME_DELAY) => {}
        }

        frame_index = (frame_index + 1) % BRAILLE_FRAMES.len();
    }

    let _ = execute!(stdout(), cursor::Show);
}

pub fn format_elapsed(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    if total_secs == 0 {
        let millis = duration.subsec_millis();
        return format!("{millis}ms");
    }

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }

    parts.join(" ")
}
