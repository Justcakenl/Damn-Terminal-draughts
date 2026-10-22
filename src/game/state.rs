use crate::game::board::Board;
use crate::game::piece::{PieceColor, PieceType};
use crate::game::rules::{Move, Rules};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GameState {
    pub board: Board,
    pub current_player: PieceColor,
    pub turn_number: u32,
    pub history: Vec<Move>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_player: PieceColor::White,
            turn_number: 1,
            history: Vec::new(),
        }
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        Rules::get_legal_moves(&self.board, self.current_player)
    }

    pub fn make_move(&mut self, m: Move) {
        let piece = self.board.get(m.from).cloned().expect("Piece must exist at 'from'");
        
        // 1. Clear from and captured
        self.board.clear(m.from);
        for cap_pos in &m.captured {
            self.board.clear(*cap_pos);
        }

        // 2. Determine final piece type (Promotion)
        let mut final_piece = piece;
        if final_piece.piece_type == PieceType::Man {
            if (final_piece.color == PieceColor::White && m.to.y == 0) || 
               (final_piece.color == PieceColor::Black && m.to.y == 9) {
                final_piece.piece_type = PieceType::King;
            }
        }

        // 3. Set at to
        self.board.set(m.to, Some(final_piece));
        
        self.history.push(m);
        self.current_player = self.current_player.opposite();
        self.turn_number += 1;
    }
}
