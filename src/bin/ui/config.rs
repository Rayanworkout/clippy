const MAX_ENTRY_DISPLAY_LENGTH: usize = 100;
const MINIMIZE_ON_COPY: bool = true;
const MINIMIZE_ON_CLEAR: bool = true;

#[derive(Clone)]
pub struct AppConfig {
    pub dark_mode: bool,
    pub max_entry_display_length: usize,
    pub minimize_on_copy: bool,
    pub minimize_on_clear: bool,
}

impl AppConfig {
    pub fn new() -> Self {
        AppConfig {
            dark_mode: true,
            max_entry_display_length: MAX_ENTRY_DISPLAY_LENGTH,
            minimize_on_copy: MINIMIZE_ON_COPY,
            minimize_on_clear: MINIMIZE_ON_CLEAR,
        }
    }
}
