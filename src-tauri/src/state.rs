use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

pub struct AppState {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}
