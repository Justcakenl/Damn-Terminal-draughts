use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, MouseButton, MouseEventKind, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::game::board::Pos;
use crate::game::piece::PieceColor;
use crate::game::rules::Move;
use crate::game::state::GameState;
use crate::ui::theme::Theme;

pub struct App {
    pub state: GameState,
    pub theme: Theme,
    cursor: Pos,
    source: Option<Pos>,
    legal_moves: Vec<Move>,
    candidate: usize,
    message: String,
    dragging: bool,
    hints: bool,
    paused: bool,
    help: bool,
    prompt: Option<Prompt>,
    undo_stack: Vec<GameState>,
    quit: bool,
    board_area: Rect,
    ai_mode: AiMode,
    ai_depth: u8,
    ai_due: Option<Instant>,
    mode_menu: bool,
    game_over: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AiMode { Human, Easy, Medium, Hard, Impossible, Auto }

#[derive(Clone, Copy)]
enum Prompt { Quit, Restart }

impl App {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            theme: Theme::default(),
            cursor: Pos::new(1, 0),
            source: None,
            legal_moves: Vec::new(),
            candidate: 0,
            message: String::from("Select a piece, then select its destination."),
            dragging: false,
            hints: true,
            paused: false,
            help: false,
            prompt: None,
            undo_stack: Vec::new(),
            quit: false,
            board_area: Rect { x: 0, y: 0, width: 0, height: 0 },
            ai_mode: AiMode::Medium,
            ai_depth: 3,
            ai_due: None,
            mode_menu: true,
            game_over: None,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        let result = self.event_loop(&mut terminal);
        // Always restore the user's terminal, including on draw/input errors.
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        result
    }

    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            self.run_ai_if_due();
            if !event::poll(Duration::from_millis(50))? { continue; }
            let event = event::read()?;
            if let Event::Mouse(mouse) = event {
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        self.dragging = true;
                        self.click_board(mouse.column, mouse.row, false);
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        // Dragging only moves the cursor. A move is committed on release.
                        if self.dragging { self.click_board(mouse.column, mouse.row, false); }
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        if self.dragging { self.click_board(mouse.column, mouse.row, true); }
                        self.dragging = false;
                    }
                    _ => {}
                }
                continue;
            }
            let Event::Key(key) = event else { continue };
            if self.mode_menu { self.handle_mode_menu(key.code); continue; }
            if self.game_over.is_some() {
                match key.code { KeyCode::Char('r') | KeyCode::Char('R') => { self.game_over = None; self.reset(); }, KeyCode::Char('q') | KeyCode::Esc => self.quit = true, _ => {} }
                if self.quit { break; }
                continue;
            }
            if self.prompt.is_some() { self.handle_prompt(key.code); if self.quit { break; } continue; }
            if self.help { if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) { self.help = false; } continue; }
            if self.paused { if matches!(key.code, KeyCode::Char('p') | KeyCode::Esc) { self.paused = false; } continue; }
            match key.code {
                KeyCode::Char('q') => self.prompt = Some(Prompt::Quit),
                KeyCode::Esc => self.cancel_selection(),
                KeyCode::Char('?') => self.help = true,
                KeyCode::Char('h') | KeyCode::Char('H') => self.hints = !self.hints,
                KeyCode::Char('p') | KeyCode::Char('P') => self.paused = true,
                KeyCode::Up | KeyCode::Char('w') => self.move_cursor(0, -1),
                KeyCode::Down | KeyCode::Char('s') => self.move_cursor(0, 1),
                KeyCode::Left | KeyCode::Char('a') => self.move_cursor(-1, 0),
                KeyCode::Right | KeyCode::Char('d') => self.move_cursor(1, 0),
                KeyCode::Enter | KeyCode::Char(' ') => self.confirm(),
                KeyCode::Tab => self.next_candidate(),
                KeyCode::Char('u') | KeyCode::Char('U') => self.undo(),
                KeyCode::Char('r') | KeyCode::Char('R') => self.prompt = Some(Prompt::Restart),
                KeyCode::Char('S') => { let _ = crate::persistence::save::save(&self.state); self.message = "Game saved.".into(); }
                KeyCode::Char('L') => if let Ok(state) = crate::persistence::save::load() { self.state = state; self.source = None; self.legal_moves.clear(); self.message = "Game loaded.".into(); },
                KeyCode::Char('1') => self.set_ai(AiMode::Human, 0),
                KeyCode::Char('2') => self.set_ai(AiMode::Easy, 2),
                KeyCode::Char('3') => self.set_ai(AiMode::Medium, 3),
                KeyCode::Char('4') => self.set_ai(AiMode::Hard, 5),
                KeyCode::Char('5') => self.set_ai(AiMode::Impossible, 7),
                KeyCode::Char('0') => self.set_ai(AiMode::Auto, 5),
                KeyCode::Char('A') => self.play_ai_if_needed(),
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_mode_menu(&mut self, code: KeyCode) {
        let selected = match code {
            KeyCode::Char('1') => Some((AiMode::Human, 0)),
            KeyCode::Char('2') => Some((AiMode::Easy, 2)),
            KeyCode::Char('3') => Some((AiMode::Medium, 3)),
            KeyCode::Char('4') => Some((AiMode::Hard, 5)),
            KeyCode::Char('5') => Some((AiMode::Impossible, 7)),
            KeyCode::Char('0') => Some((AiMode::Auto, 5)),
            KeyCode::Char('q') | KeyCode::Esc => { self.quit = true; None },
            _ => None,
        };
        if let Some((mode, depth)) = selected {
            self.ai_mode = mode;
            self.ai_depth = depth;
            self.mode_menu = false;
            self.message = format!("Mode selected: {}", self.mode_name());
        }
    }

    fn click_board(&mut self, column: u16, row: u16, release: bool) {
        // Convert mouse coordinates using the actual rendered board rectangle.
        let area = self.board_area;
        if column < area.x + 1 || row < area.y + 1 || column >= area.x + area.width.saturating_sub(1) || row >= area.y + area.height.saturating_sub(1) { return; }
        let inner_x = column - area.x - 1;
        let inner_y = row - area.y - 1;
        let cell_w = (area.width.saturating_sub(2) / 10).max(1);
        let cell_h = (area.height.saturating_sub(2) / 10).max(1);
        let x = (inner_x / cell_w).min(9) as u8;
        let y = (inner_y / cell_h).min(9) as u8;
        let pos = Pos::new(x, y);
        if !pos.is_playable() { return; }
        self.cursor = pos;
        // Dragging starts on a source square and commits only on release at a legal destination.
        if release && self.source.is_some() {
            self.confirm();
        } else if !release && self.source.is_none() {
            // Mouse down selects the piece; drag release performs the destination move.
            self.confirm();
        }
    }

    fn move_cursor(&mut self, dx: i8, dy: i8) {
        let x = (self.cursor.x as i8 + dx).clamp(0, 9) as u8;
        let y = (self.cursor.y as i8 + dy).clamp(0, 9) as u8;
        let next = Pos::new(x, y);
        if next.is_playable() { self.cursor = next; }
    }

    fn check_game_over(&mut self) {
        if self.state.get_legal_moves().is_empty() { self.finish_game(); }
    }

    fn finish_game(&mut self) {
        let winner = match self.state.current_player { PieceColor::White => "BLACK", PieceColor::Black => "WHITE" };
        self.game_over = Some(format!("{} WINS!\n\nNo legal moves remain.", winner));
    }

    fn set_ai(&mut self, mode: AiMode, depth: u8) {
        self.ai_mode = mode;
        self.ai_depth = depth;
        self.ai_due = None;
        self.reset();
        self.message = format!("Mode: {}", self.mode_name());
    }

    fn mode_name(&self) -> &'static str {
        match self.ai_mode { AiMode::Human => "Local 2-player", AiMode::Easy => "AI Easy", AiMode::Medium => "AI Medium", AiMode::Hard => "AI Hard", AiMode::Impossible => "AI Impossible", AiMode::Auto => "Computer vs Computer" }
    }

    fn run_ai_if_due(&mut self) {
        let ai_turn = match self.ai_mode {
            AiMode::Human => false,
            AiMode::Auto => true,
            _ => self.state.current_player == PieceColor::Black,
        };
        if !ai_turn { return; }
        if self.ai_due.map(|due| Instant::now() < due).unwrap_or(false) { return; }
        self.play_ai_if_needed();
    }

    fn play_ai_if_needed(&mut self) {
        // Local mode is strictly human-vs-human: never invoke the AI there.
        if matches!(self.ai_mode, AiMode::Human) { return; }
        if self.state.get_legal_moves().is_empty() { self.finish_game(); return; }
        let cancel = crate::ai::search::cancellation_flag();
        if let Some(mv) = crate::ai::search::best_move(&self.state, self.ai_depth, &cancel) {
            self.undo_stack.push(self.state.clone());
            self.state.make_move(mv);
            self.ai_due = Some(Instant::now() + Duration::from_millis(180));
            self.message = format!("{} is thinking...", self.mode_name());
            self.check_game_over();
        } else { self.finish_game(); }
    }

    fn cancel_selection(&mut self) {
        if self.source.is_some() { self.source = None; self.legal_moves.clear(); self.message = "Selection cancelled.".into(); }
        else { self.message = "Press Q again? Use Q for quit confirmation.".into(); }
    }

    fn handle_prompt(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => match self.prompt.take() {
                Some(Prompt::Quit) => self.quit = true,
                Some(Prompt::Restart) => self.reset(), None => {}
            },
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => self.prompt = None,
            _ => {}
        }
    }

    fn confirm(&mut self) {
        if self.source.is_some() {
            if let Some(mv) = self.legal_moves.iter().find(|m| m.to == self.cursor).cloned() {
                self.undo_stack.push(self.state.clone());
                self.state.make_move(mv);
                self.source = None;
                self.legal_moves.clear();
                self.candidate = 0;
                self.message = if self.ai_mode == AiMode::Human {
                    format!("{:?} moved. Your turn.", self.state.current_player.opposite())
                } else {
                    "Move played. Computer is thinking...".into()
                };
                self.play_ai_if_needed();
            } else if self.state.board.get(self.cursor).map(|p| p.color == self.state.current_player).unwrap_or(false) {
                let moves: Vec<Move> = self.state.get_legal_moves().into_iter().filter(|m| m.from == self.cursor).collect();
                if !moves.is_empty() {
                    self.source = Some(self.cursor);
                    self.legal_moves = moves;
                    self.candidate = 0;
                    self.message = "Piece switched. Choose a cyan destination.".into();
                } else {
                    self.message = "That piece has no legal move.".into();
                }
            } else {
                self.message = "Illegal destination. Choose a cyan square or press Esc.".into();
            }
        } else {
            let moves: Vec<Move> = self.state.get_legal_moves().into_iter()
                .filter(|m| m.from == self.cursor).collect();
            if moves.is_empty() {
                self.message = if self.state.board.get(self.cursor).is_some() {
                    "That piece has no legal move (or is not yours).".into()
                } else { "Select one of your pieces.".into() };
            } else {
                self.source = Some(self.cursor);
                self.legal_moves = moves;
                self.candidate = 0;
                self.message = "Choose a highlighted destination and press Enter.".into();
            }
        }
    }

    fn next_candidate(&mut self) {
        if self.legal_moves.is_empty() { return; }
        self.candidate = (self.candidate + 1) % self.legal_moves.len();
        self.cursor = self.legal_moves[self.candidate].to;
    }

    fn undo(&mut self) {
        if self.source.is_some() { self.cancel_selection(); return; }
        if let Some(state) = self.undo_stack.pop() { self.state = state; self.message = "Move undone.".into(); }
        else { self.message = "Nothing to undo.".into(); }
    }

    fn reset(&mut self) {
        self.state = GameState::new();
        self.undo_stack.clear();
        self.source = None;
        self.legal_moves.clear();
        self.cursor = Pos::new(1, 0);
        self.game_over = None;
        self.message = "New game started.".into();
    }

    fn render(&mut self, frame: &mut Frame) {
        let size = frame.size();
        // The mode screen is a separate first screen: do not draw the board behind it.
        if self.mode_menu {
            let text = "DAMN\n\nCHOOSE GAME MODE\n\n1  Local two-player\n2  Human vs AI (Easy)\n3  Human vs AI (Medium)\n4  Human vs AI (Hard)\n5  Human vs AI (Impossible)\n0  Computer vs computer\n\nPress a number to start\nQ  Quit";
            frame.render_widget(Paragraph::new(text).block(Block::default().title(" DAMN — New Game ").borders(Borders::ALL)), size);
            return;
        }
        if let Some(result) = &self.game_over {
            let text = format!("DAMN\n\n{}\n\nR  Rematch\nQ  Quit", result);
            frame.render_widget(Paragraph::new(text).block(Block::default().title(" GAME OVER ").borders(Borders::ALL)), size);
            return;
        }
        // Help is a full-screen menu, rendered before the board so it cannot sit behind it.
        if self.help {
            let text = "DAMN — CONTROLS\n\nNAVIGATION\n  ↑ ↓ ← → / W A S D   Move cursor\n\nPLAYING\n  Enter / Space        Select piece or confirm move\n  Tab                  Cycle legal destinations\n  Esc                  Cancel current selection\n  H                    Toggle legal-move hints\n  Mouse click          Select or confirm\n  Mouse drag           Drag a piece to a legal square\n\nGAME\n  U                    Undo last move\n  R                    Restart (confirmation required)\n  P                    Pause\n  S / L                Save / load game\n  Q                    Quit (confirmation required)\n\nMODES\n  1 Local  2 Easy  3 Medium  4 Hard  5 Impossible  0 Auto\n\nCyan = legal destination   Yellow = selected piece\nPress ? or Esc to close this menu";
            frame.render_widget(Paragraph::new(text).block(Block::default().title(" Help ").borders(Borders::ALL)), size);
            return;
        }
        // Compact mode keeps the board usable on small terminals and moves status below it.
        let compact = size.width < 90 || size.height < 28;
        if compact && (size.width < 42 || size.height < 16) {
            frame.render_widget(Paragraph::new("DAMN\n\nTerminal too small.\nMinimum: 42x16\nPlease enlarge the window.").block(Block::default().borders(Borders::ALL)), size);
            return;
        }
        if compact {
            let rows = Layout::default().direction(Direction::Vertical)
                .constraints([Constraint::Length(size.height.saturating_sub(7)), Constraint::Min(7)]).split(size);
            self.board_area = rows[0];
            self.render_board(frame, rows[0]);
            let compact_status = format!("{} | Turn: {:?} | W {}  B {}\nArrows/WASD move · Enter/Space select · Esc cancel · ? help · Q quit\n{}", self.mode_name(), self.state.current_player, self.count(PieceColor::White), self.count(PieceColor::Black), self.message);
            frame.render_widget(Paragraph::new(compact_status).block(Block::default().borders(Borders::ALL)), rows[1]);
            return;
        }
        // Give the board all available width up to a readable side panel. It scales with fullscreen.
        let board_width = ((size.height.saturating_sub(2) / 2) * 6 + 2).min(size.width.saturating_sub(28)).max(62);
        let columns = Layout::default().direction(Direction::Horizontal)
            .constraints([Constraint::Length(board_width), Constraint::Min(28)]).split(size);
        self.board_area = columns[0];
        self.render_board(frame, columns[0]);
        let white_left = (0..10).flat_map(|y| (0..10).map(move |x| Pos::new(x, y))).filter(|&p| self.state.board.get(p).map(|q| q.color == PieceColor::White).unwrap_or(false)).count();
        let black_left = (0..10).flat_map(|y| (0..10).map(move |x| Pos::new(x, y))).filter(|&p| self.state.board.get(p).map(|q| q.color == PieceColor::Black).unwrap_or(false)).count();
        let history = self.state.history.iter().rev().take(12).enumerate()
            .map(|(i, m)| Line::from(format!("{}. {}:{} → {}:{}", i + 1, m.from.x, m.from.y, m.to.x, m.to.y)))
            .collect::<Vec<_>>();
        let side = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(5)]).split(columns[1]);
        if self.paused {
            frame.render_widget(Paragraph::new("PAUSED\n\nPress P or Esc to resume\nQ to quit").block(Block::default().title(" Pause ").borders(Borders::ALL)), size);
            return;
        }
        if let Some(prompt) = self.prompt {
            let text = match prompt { Prompt::Quit => "Quit DAMN?  [Y]es / [N]o", Prompt::Restart => "Restart game?  [Y]es / [N]o" };
            frame.render_widget(Paragraph::new(text).block(Block::default().title(" Confirm ").borders(Borders::ALL)), size);
            return;
        }
        let status = vec![
            Line::from(Span::styled("DAMN", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(match self.ai_mode { AiMode::Human => "MODE: LOCAL 2-PLAYER", AiMode::Auto => "MODE: COMPUTER VS COMPUTER", _ => "MODE: HUMAN VS AI" }),
            Line::from(match self.ai_mode {
                AiMode::Human => "WHITE: W   BLACK: B   BOTH HUMAN",
                AiMode::Auto => "WHITE: W   BLACK: B   BOTH COMPUTER",
                _ => "YOU: WHITE (W)   COMPUTER: BLACK (B)",
            }),
            Line::from(format!("TURN: {:?}  MOVE #{}", self.state.current_player, self.state.turn_number)),
            Line::from(format!("SCORE  WHITE: {:02}   BLACK: {:02}", white_left, black_left)),
            Line::from("WHITE moves first"),
            Line::from("[W]/[B] = man   [W*]/[B*] = king"),
            Line::from(""),
            Line::from(self.message.as_str()),
            Line::from(""),
            Line::from("CONTROLS   [?] full help menu"),
            Line::from("↑↓←→ / WASD  Move cursor"),
            Line::from("Enter         Select / play"),
            Line::from("Space/Tab     Cycle destinations"),
            Line::from("Mouse click   Select / play"),
            Line::from("Mouse drag    Drag piece to square"),
            Line::from("U undo  R restart  Q/Esc quit"),
            Line::from(format!("MODE: {}", self.mode_name())),
            Line::from("1 local  2 easy  3 medium"),
            Line::from("4 hard  5 impossible  0 auto"),
            Line::from("S save  L load  A force AI"),
            Line::from("?  CONTROLS / HELP MENU"),
        ];
        frame.render_widget(Paragraph::new(status).block(Block::default().title(" Status ").borders(Borders::ALL)), side[0]);
        frame.render_widget(Paragraph::new(history).block(Block::default().title(" Move history ").borders(Borders::ALL)), side[1]);
    }

    fn count(&self, color: PieceColor) -> usize {
        (0..10).flat_map(|y| (0..10).map(move |x| Pos::new(x, y)))
            .filter(|&pos| self.state.board.get(pos).map(|p| p.color == color).unwrap_or(false)).count()
    }

    fn render_board(&self, frame: &mut Frame, area: Rect) {
        let board_block = Block::default().title(" International draughts ").borders(Borders::ALL);
        let inner = board_block.inner(area);
        frame.render_widget(board_block, area);
        let cell_w = (inner.width / 10).max(1);
        let cell_h = (inner.height / 10).max(1);
        for y in 0..10u8 { for x in 0..10u8 {
            let pos = Pos::new(x, y);
            let rect = Rect { x: inner.x + u16::from(x) * cell_w, y: inner.y + u16::from(y) * cell_h, width: cell_w, height: cell_h };
            let dark = (x + y) % 2 == 1;
            let mut style = Style::default().bg(if dark { self.theme.dark_square } else { self.theme.light_square });
                if self.source == Some(pos) { style = style.bg(Color::Yellow).add_modifier(Modifier::BOLD); }
            if self.hints && self.legal_moves.iter().any(|m| m.to == pos) { style = style.bg(Color::Cyan); }
            if self.cursor == pos { style = style.bg(Color::Magenta).add_modifier(Modifier::BOLD); }
            if pos.is_playable() && self.state.current_player == PieceColor::White && self.state.board.get(pos).map(|p| p.color == PieceColor::White).unwrap_or(false) {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            let glyph = self.state.board.get(pos).map(|p| match (p.color, p.is_king()) {
                (PieceColor::White, false) => " W ",
                (PieceColor::Black, false) => " B ",
                (PieceColor::White, true) => " W*",
                (PieceColor::Black, true) => " B*",
            }).unwrap_or("   ");
            let piece_style = match self.state.board.get(pos).map(|p| p.color) {
                Some(PieceColor::White) => style.fg(self.theme.white_piece).add_modifier(Modifier::BOLD),
                Some(PieceColor::Black) => style.fg(self.theme.black_piece).add_modifier(Modifier::BOLD),
                None => style,
            };
            frame.render_widget(Paragraph::new(glyph).style(piece_style), rect);
        }}
    }
}
