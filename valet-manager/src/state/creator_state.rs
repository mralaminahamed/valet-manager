use std::collections::HashMap;
use crate::creator::output_streamer::OutputLine;
use crate::creator::project_types::ProjectType;

#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub enum CreatorStep {
    #[default]
    SelectType,
    Configure,
    PostInstall,
    Progress,
    Complete,
    Error,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CreatorState {
    pub step: CreatorStep,
    pub selected_type: Option<ProjectType>,
    pub form_values: HashMap<String, String>,    // option key → user value
    pub output_lines: Vec<OutputLine>,           // streamed terminal output
    pub error: Option<String>,                   // last error message
    pub is_running: bool,                        // install in progress
    pub cancel_requested: bool,
    // Wizard UI state
    pub active_group: crate::creator::project_types::ProjectGroup,
    pub post_install_link: bool,
    pub post_install_secure: bool,
    pub post_install_isolate: bool,
    pub isolate_php_version: Option<String>,
    pub post_install_open_browser: bool,
    pub post_install_open_editor: bool,
}

impl Default for CreatorState {
    fn default() -> Self {
        Self {
            step: CreatorStep::default(),
            selected_type: None,
            form_values: HashMap::new(),
            output_lines: Vec::new(),
            error: None,
            is_running: false,
            cancel_requested: false,
            active_group: crate::creator::project_types::ProjectGroup::WordPress,
            post_install_link: true,
            post_install_secure: true,
            post_install_isolate: false,
            isolate_php_version: None,
            post_install_open_browser: false,
            post_install_open_editor: false,
        }
    }
}

impl CreatorState {
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    #[allow(dead_code)]
    pub fn go_to(&mut self, step: CreatorStep) {
        self.step = step;
    }

    #[allow(dead_code)]
    pub fn add_output_line(&mut self, line: OutputLine) {
        if self.output_lines.len() >= 1000 {
            self.output_lines.remove(0);
        }
        self.output_lines.push(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creator_state_defaults_to_select_type() {
        let state = CreatorState::default();
        assert_eq!(state.step, CreatorStep::SelectType);
        assert!(state.selected_type.is_none());
        assert!(state.form_values.is_empty());
        assert!(state.output_lines.is_empty());
        assert!(!state.is_running);
    }

    #[test]
    fn creator_state_reset_clears_all() {
        let mut state = CreatorState::default();
        state.step = CreatorStep::Progress;
        state.is_running = true;
        state.reset();
        assert_eq!(state.step, CreatorStep::SelectType);
        assert!(!state.is_running);
    }

    #[test]
    fn creator_state_add_output_line_caps_at_1000() {
        use crate::creator::output_streamer::{OutputLine, Stream};
        let mut state = CreatorState::default();
        for i in 0..1001 {
            state.add_output_line(OutputLine {
                text: format!("line {}", i),
                stream: Stream::Stdout,
                timestamp: chrono::Local::now(),
            });
        }
        assert_eq!(state.output_lines.len(), 1000);
    }

    #[test]
    fn creator_step_go_to() {
        let mut state = CreatorState::default();
        state.go_to(CreatorStep::Configure);
        assert_eq!(state.step, CreatorStep::Configure);
    }
}
