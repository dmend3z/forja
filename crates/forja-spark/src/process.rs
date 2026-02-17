use std::path::PathBuf;
use std::sync::Arc;

use tokio::process::Command;
use tokio::sync::{RwLock, broadcast};

use crate::events::{SparkEvent, SparkStatus};

/// A managed `claude` CLI process that streams chat events.
///
/// Each Spark represents one conversation session. The first message
/// spawns a new claude process; subsequent messages use `--resume`
/// with the captured session ID for multi-turn conversation.
pub struct Spark {
    pub id: String,
    pub project_id: String,
    pub session_id: Arc<RwLock<Option<String>>>,
    pub status: Arc<RwLock<SparkStatus>>,
    pub events_tx: broadcast::Sender<SparkEvent>,
    pub cwd: PathBuf,
}

impl Spark {
    /// Spawn a new spark with the given initial prompt.
    ///
    /// Launches `claude --output-format stream-json --print -- <prompt>` in
    /// the project's working directory. Streams events via `subscribe()`.
    ///
    /// # Implementation note
    ///
    /// The actual process spawning, stdout parsing, and session ID capture
    /// will be implemented in Phase 4 after investigating the real
    /// stream-json output format.
    pub async fn spawn(project_id: String, prompt: String, cwd: PathBuf) -> crate::Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let (events_tx, _) = broadcast::channel(256);
        let status = Arc::new(RwLock::new(SparkStatus::Starting));
        let session_id = Arc::new(RwLock::new(None));

        let spark = Self {
            id: id.clone(),
            project_id,
            session_id: session_id.clone(),
            status: status.clone(),
            events_tx: events_tx.clone(),
            cwd: cwd.clone(),
        };

        // TODO (Phase 4): Spawn the claude process, pipe stdout through
        // parser::parse_line, and emit ChatEvents on events_tx.
        //
        // Pseudocode:
        // let mut child = Command::new("claude")
        //     .args(["--output-format", "stream-json", "--print", "--", &prompt])
        //     .current_dir(&cwd)
        //     .stdout(Stdio::piped())
        //     .stderr(Stdio::piped())
        //     .spawn()?;
        //
        // tokio::spawn(async move {
        //     // Read stdout line by line
        //     // Parse each line with parser::parse_line
        //     // Capture session_id from output
        //     // Emit SparkEvent::Chat on events_tx
        //     // Update status on completion
        // });

        // Suppress unused variable warnings in skeleton
        let _ = prompt;
        let _ = Command::new("claude");

        Ok(spark)
    }

    /// Send a follow-up message to an existing spark (multi-turn).
    ///
    /// Uses `claude --output-format stream-json --print --resume <session_id> -- <prompt>`.
    pub async fn send_message(&self, prompt: String) -> crate::Result<()> {
        let _session = self.session_id.read().await;
        // TODO (Phase 4): Spawn a new claude process with --resume <session_id>
        let _ = prompt;
        Ok(())
    }

    /// Subscribe to this spark's event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<SparkEvent> {
        self.events_tx.subscribe()
    }

    /// Get the current status.
    pub async fn status(&self) -> SparkStatus {
        *self.status.read().await
    }
}
