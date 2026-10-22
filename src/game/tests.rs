#[cfg(test)]
mod tests {
    use crate::game::board::{Board, Pos};
    use crate::game::piece::{Piece, PieceColor, PieceType};
    use crate::game::rules::Rules;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn test_initial_board() {
        let board = Board::new();
        let mut white_count = 0;
        let mut black_count = 0;
        for y in 0..10 {
            for x in 0..10 {
                if let Some(p) = board.get(Pos::new(x, y)) {
                    if p.color == PieceColor::White { white_count += 1; }
                    else { black_count += 1; }
                }
            }
        }
        assert_eq!(white_count, 20);
        assert_eq!(black_count, 20);
    }

    #[test]
    fn test_mandatory_capture() {
        let mut board = Board::new();
        board.squares = [[None; 10]; 10];
        
        // Setup: White man at (4,4) must capture Black man at (5,3)
        board.set(Pos::new(4, 4), Some(Piece::new(PieceColor::White, PieceType::Man)));
        board.set(Pos::new(5, 3), Some(Piece::new(PieceColor::Black, PieceType::Man)));
        // Square (6, 2) is empty
        
        let moves = Rules::get_legal_moves(&board, PieceColor::White);
        assert_eq!(moves.len(), 1, "Should only have the capture move");
        assert_eq!(moves[0].to, Pos::new(6, 2));
        assert_eq!(moves[0].captured.len(), 1);
    }

    #[test]
    fn test_maximal_capture() {
        let mut board = Board::new();
        board.squares = [[None; 10]; 10];
        
        // White man at (4,4)
        board.set(Pos::new(4, 4), Some(Piece::new(PieceColor::White, PieceType::Man)));
        
        // Option 1: Capture 1 piece: jump (3,3) to (2,2)
        board.set(Pos::new(3, 3), Some(Piece::new(PieceColor::Black, PieceType::Man)));
        
        // Option 2: Capture 2 pieces: jump (5,3) to (6,2) then jump (7,1) to (8,0)
        board.set(Pos::new(5, 3), Some(Piece::new(PieceColor::Black, PieceType::Man)));
        board.set(Pos::new(7, 1), Some(Piece::new(PieceColor::Black, PieceType::Man)));
        
        let moves = Rules::get_legal_moves(&board, PieceColor::White);
        
        assert_eq!(moves.len(), 1, "Only the maximal capture sequence should be returned");
        assert_eq!(moves[0].captured.len(), 2);
        assert_eq!(moves[0].to, Pos::new(8, 0));
    }

    #[test]
    fn ai_always_returns_a_legal_move() {
        let state = crate::game::state::GameState::new();
        let cancel = AtomicBool::new(false);
        let selected = crate::ai::search::best_move(&state, 2, &cancel).expect("opening has moves");
        assert!(state.get_legal_moves().contains(&selected));
    }

    #[test]
    fn ai_can_respond_after_a_human_move() {
        let mut state = crate::game::state::GameState::new();
        let first = state.get_legal_moves().into_iter().next().expect("opening move");
        state.make_move(first);
        let cancel = AtomicBool::new(false);
        let reply = crate::ai::search::best_move(&state, 2, &cancel).expect("black has moves");
        assert!(state.get_legal_moves().contains(&reply));
    }
}
