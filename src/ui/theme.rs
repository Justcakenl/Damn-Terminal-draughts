
pub struct Theme {
    pub light_square: ratatui::style::Color,
    pub dark_square: ratatui::style::Color,
    pub white_piece: ratatui::style::Color,
    pub black_piece: ratatui::style::Color,
    #[allow(dead_code)]
    pub highlight_color: ratatui::style::Color,
    #[allow(dead_code)]
    pub legal_move_color: ratatui::style::Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            light_square: ratatui::style::Color::Rgb(240, 240, 240),
            dark_square: ratatui::style::Color::Rgb(100, 100, 100),
            white_piece: ratatui::style::Color::White,
            black_piece: ratatui::style::Color::Black,
            highlight_color: ratatui::style::Color::Yellow,
            legal_move_color: ratatui::style::Color::Cyan,
        }
    }
}
