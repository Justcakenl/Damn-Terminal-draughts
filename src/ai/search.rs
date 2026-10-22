use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use crate::game::piece::PieceColor;
use crate::game::rules::Move;
use crate::game::state::GameState;

use super::evaluation::evaluate;

/// Deterministic alpha-beta search. Every returned move comes from the rules engine.
pub fn best_move(state: &GameState, depth: u8, cancel: &AtomicBool) -> Option<Move> {
    let moves = state.get_legal_moves();
    let root = state.current_player;
    let mut best = None;
    let mut best_score = i32::MIN;
    let mut alpha = i32::MIN + 1;

    for mv in moves {
        if cancel.load(Ordering::Relaxed) { return best; }
        let mut child = state.clone();
        child.make_move(mv.clone());
        let score = minimax(&child, depth.saturating_sub(1), alpha, i32::MAX, root, cancel);
        if score > best_score {
            best_score = score;
            best = Some(mv);
        }
        alpha = alpha.max(score);
    }
    best
}

fn minimax(state: &GameState, depth: u8, mut alpha: i32, mut beta: i32, root: PieceColor, cancel: &AtomicBool) -> i32 {
    if cancel.load(Ordering::Relaxed) { return 0; }
    let moves = state.get_legal_moves();
    if moves.is_empty() {
        return if state.current_player == root { -100_000 - i32::from(depth) } else { 100_000 + i32::from(depth) };
    }
    if depth == 0 { return evaluate(&state.board, root); }

    let maximizing = state.current_player == root;
    if maximizing {
        let mut value = i32::MIN + 1;
        for mv in moves {
            let mut child = state.clone();
            child.make_move(mv);
            value = value.max(minimax(&child, depth - 1, alpha, beta, root, cancel));
            alpha = alpha.max(value);
            if alpha >= beta { break; }
        }
        value
    } else {
        let mut value = i32::MAX;
        for mv in moves {
            let mut child = state.clone();
            child.make_move(mv);
            value = value.min(minimax(&child, depth - 1, alpha, beta, root, cancel));
            beta = beta.min(value);
            if alpha >= beta { break; }
        }
        value
    }
}

pub fn cancellation_flag() -> Arc<AtomicBool> { Arc::new(AtomicBool::new(false)) }
