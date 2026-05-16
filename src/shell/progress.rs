use std::time::{Duration, Instant};

use super::format::{self, StepStatus};

#[derive(Debug, Clone)]
pub(super) struct ProgressTracker {
    start: Instant,
    steps: Vec<Step>,
}

#[derive(Debug, Clone)]
struct Step {
    name: String,
    status: StepStatus,
    started_at: Option<Instant>,
    finished_at: Option<Instant>,
}

impl ProgressTracker {
    pub(super) fn new(names: &[&str]) -> Self {
        Self {
            start: Instant::now(),
            steps: names
                .iter()
                .map(|name| Step {
                    name: (*name).to_string(),
                    status: StepStatus::Pending,
                    started_at: None,
                    finished_at: None,
                })
                .collect(),
        }
    }

    pub(super) fn start_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.status = StepStatus::Running;
            step.started_at = Some(Instant::now());
        }
    }

    pub(super) fn complete_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.status = StepStatus::Done;
            step.finished_at = Some(Instant::now());
        }
    }

    pub(super) fn fail_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.status = StepStatus::Failed;
            step.finished_at = Some(Instant::now());
        }
    }

    pub(super) fn render(&self) -> String {
        let mut lines = vec!["DAG".to_string()];
        for step in &self.steps {
            lines.push(format::progress_step(
                &elapsed_label(step.started_at, step.finished_at),
                step.status,
                &step.name,
            ));
        }
        lines.join("\n")
    }

    pub(super) fn finish(&self) -> Duration {
        self.start.elapsed()
    }
}

pub(super) fn default_run_tracker() -> ProgressTracker {
    let mut tracker = ProgressTracker::new(&[
        "prepare workspace",
        "build context",
        "execute",
        "diff guard",
        "verify",
        "commit",
    ]);
    tracker.start_step(0);
    tracker
}

fn elapsed_label(started: Option<Instant>, finished: Option<Instant>) -> String {
    let Some(started) = started else {
        return "--:--".to_string();
    };
    let elapsed = finished
        .unwrap_or_else(Instant::now)
        .duration_since(started);
    format!(
        "{:02}:{:02}",
        elapsed.as_secs() / 60,
        elapsed.as_secs() % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_named_progress_steps() {
        let mut tracker = ProgressTracker::new(&["prepare", "verify"]);
        tracker.start_step(0);
        tracker.complete_step(0);
        tracker.start_step(1);

        let output = tracker.render();

        assert!(output.contains("prepare"));
        assert!(output.contains("verify"));
        assert!(output.contains("[ok]"));
        assert!(output.contains("[run]"));
    }
}
