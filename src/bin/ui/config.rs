use serde::{Deserialize, Serialize};

const DEFAULT_MAX_ENTRY_DISPLAY_LENGTH: usize = 100;
const DEFAULT_MINIMIZE_ON_COPY: bool = true;
const DEFAULT_MINIMIZE_ON_CLEAR: bool = true;

#[derive(Clone, Serialize, Deserialize)]
pub struct ClippyConfig {
    pub dark_mode: bool,
    pub max_entry_display_length: usize,
    pub minimize_on_copy: bool,
    pub minimize_on_clear: bool,
}

impl Default for ClippyConfig {
    fn default() -> Self {
        Self {
            dark_mode: true,
            max_entry_display_length: DEFAULT_MAX_ENTRY_DISPLAY_LENGTH,
            minimize_on_copy: DEFAULT_MINIMIZE_ON_COPY,
            minimize_on_clear: DEFAULT_MINIMIZE_ON_CLEAR,
        }
    }
}
