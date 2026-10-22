use crate::game::board::Board;
use crate::game::piece::{PieceColor, PieceType};

#[allow(dead_code)]
pub fn evaluate(board: &Board, perspective: PieceColor) -> i32 {
    let mut score = 0;
    for y in 0..10 {
        for x in 0..10 {
            if let Some(piece) = board.squares[y][x] {
                let value = match piece.piece_type { PieceType::Man => 100, PieceType::King => 300 };
                let value = if piece.color == perspective { value } else { -value };
                score += value;
            }
        }
    }
    score
}
