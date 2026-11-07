//! One background job and its result channel.
//!
//! The launcher runs at most one mutation at a time. The window stays
//! responsive and polls the channel on each frame.

use std::sync::mpsc::{self, Receiver, TryRecvError};

use crate::error::LauncherError;

/// The result of one background job.
#[derive(Debug)]
pub struct JobOutcome {
    /// Human-readable job label.
    pub label: String,
    /// Success message or typed failure.
    pub result: Result<String, LauncherError>,
}

/// A single-job worker.
#[derive(Debug, Default)]
pub struct Worker {
    receiver: Option<Receiver<JobOutcome>>,
    label: Option<String>,
}

impl Worker {
    /// Creates an idle worker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            receiver: None,
            label: None,
        }
    }

    /// Returns whether a job is running.
    #[must_use]
    pub const fn is_busy(&self) -> bool {
        self.receiver.is_some()
    }

    /// Returns the running job label.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Starts a job unless one is already running.
    pub fn start<F>(&mut self, label: impl Into<String>, job: F)
    where
        F: FnOnce() -> Result<String, LauncherError> + Send + 'static,
    {
        if self.is_busy() {
            return;
        }
        let label = label.into();
        let (sender, receiver) = mpsc::channel();
        let job_label = label.clone();
        std::thread::spawn(move || {
            let result = job();
            let _ = sender.send(JobOutcome {
                label: job_label,
                result,
            });
        });
        self.receiver = Some(receiver);
        self.label = Some(label);
    }

    /// Returns a finished job, if one is ready.
    pub fn poll(&mut self) -> Option<JobOutcome> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(outcome) => {
                self.receiver = None;
                self.label = None;
                Some(outcome)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                let label = self.label.take().unwrap_or_else(|| "operation".to_owned());
                self.receiver = None;
                Some(JobOutcome {
                    label,
                    result: Err(LauncherError::Rejected(
                        "the worker stopped without a result".to_owned(),
                    )),
                })
            }
        }
    }
}
