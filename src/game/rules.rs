use crate::game::board::{Board, Pos};
use crate::game::piece::{Piece, PieceColor};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Move {
    pub from: Pos,
    pub to: Pos,
    pub captured: Vec<Pos>,
}

impl Move {
    pub fn new(from: Pos, to: Pos) -> Self {
        Self {
            from,
            to,
            captured: Vec::new(),
        }
    }

}

pub struct Rules;

impl Rules {
    pub fn get_legal_moves(board: &Board, player: PieceColor) -> Vec<Move> {
        let mut all_captures = Vec::new();
        let mut all_simple_moves = Vec::new();

        for y in 0..10 {
            for x in 0..10 {
                let pos = Pos::new(x, y);
                if let Some(piece) = board.get(pos) {
                    if piece.color == player {
                        // Find all capture sequences for this piece
                        let piece_captures = Self::find_all_capture_sequences(board, pos);
                        all_captures.extend(piece_captures);

                        // Find all simple moves
                        all_simple_moves.extend(Self::get_simple_moves(board, pos));
                    }
                }
            }
        }

        if !all_captures.is_empty() {
            // International Draughts: Mandatory capture of the MAXIMUM number of pieces.
            let max_captured = all_captures.iter().map(|m| m.captured.len()).max().unwrap_or(0);
            all_captures.into_iter().filter(|m| m.captured.len() == max_captured).collect()
        } else {
            all_simple_moves
        }
    }

    fn find_all_capture_sequences(board: &Board, start_pos: Pos) -> Vec<Move> {
        let mut results = Vec::new();
        let piece = match board.get(start_pos) {
            Some(p) => p.clone(),
            None => return results,
        };

        let mut temp_board = board.clone();
        Self::recurse_captures(&mut temp_board, start_pos, start_pos, piece, Vec::new(), &mut results);
        
        results
    }

    fn recurse_captures(
        board: &mut Board,
        start_pos: Pos,
        curr_pos: Pos,
        piece: Piece,
        captured_so_far: Vec<Pos>,
        results: &mut Vec<Move>,
    ) {
        let captures = Self::get_single_captures(board, curr_pos, &piece);
        
        if captures.is_empty() {
            if !captured_so_far.is_empty() {
                results.push(Move {
                    from: start_pos,
                    to: curr_pos,
                    captured: captured_so_far,
                });
            }
            return;
        }

        for (to, victim) in captures {
            let mut next_captured = captured_so_far.clone();
            next_captured.push(victim);

            // Snapshot state to restore exactly
            let original_curr = board.get(curr_pos).cloned();
            let original_victim = board.get(victim).cloned();
            let original_to = board.get(to).cloned();

            board.clear(curr_pos);
            board.clear(victim);
            board.set(to, Some(piece));

            Self::recurse_captures(board, start_pos, to, piece, next_captured, results);

            // Restore precisely
            board.set(curr_pos, original_curr);
            board.set(victim, original_victim);
            board.set(to, original_to);
        }
    }

    fn get_single_captures(board: &Board, pos: Pos, piece: &Piece) -> Vec<(Pos, Pos)> {
        let mut captures = Vec::new();
        let dirs = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

        if piece.is_king() {
            for (dx, dy) in dirs {
                let mut curr_x = pos.x as i8 + dx;
                let mut curr_y = pos.y as i8 + dy;
                let mut victim_pos = None;

                while curr_x >= 0 && curr_x < 10 && curr_y >= 0 && curr_y < 10 {
                    let curr_pos = Pos::new(curr_x as u8, curr_y as u8);
                    match board.get(curr_pos) {
                        Some(p) => {
                            if p.color == piece.color { break; }
                            if victim_pos.is_some() { break; }
                            victim_pos = Some(curr_pos);
                        }
                        None => {}
                    }
                    curr_x += dx;
                    curr_y += dy;
                }

                if let Some(v_pos) = victim_pos {
                    let mut land_x = v_pos.x as i8 + dx;
                    let mut land_y = v_pos.y as i8 + dy;
                    while land_x >= 0 && land_x < 10 && land_y >= 0 && land_y < 10 {
                        let land_pos = Pos::new(land_x as u8, land_y as u8);
                        if board.get(land_pos).is_some() { break; }
                        captures.push((land_pos, v_pos));
                        land_x += dx;
                        land_y += dy;
                    }
                }
            }
        } else {
            let dys = [1, -1];
            let dxs = [1, -1];
            for dy in dys {
                for dx in dxs {
                    let v_x = pos.x as i8 + dx;
                    let v_y = pos.y as i8 + dy;
                    let l_x = pos.x as i8 + dx * 2;
                    let l_y = pos.y as i8 + dy * 2;
                    if l_x >= 0 && l_x < 10 && l_y >= 0 && l_y < 10 {
                        let v_pos = Pos::new(v_x as u8, v_y as u8);
                        let l_pos = Pos::new(l_x as u8, l_y as u8);
                        if let Some(v) = board.get(v_pos) {
                            if v.color != piece.color && board.get(l_pos).is_none() {
                                captures.push((l_pos, v_pos));
                            }
                        }
                    }
                }
            }
        }
        captures
    }

    pub fn get_simple_moves(board: &Board, pos: Pos) -> Vec<Move> {
        let mut moves = Vec::new();
        let piece = match board.get(pos) {
            Some(p) => p,
            None => return moves,
        };

        if piece.is_king() {
            let dirs = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
            for (dx, dy) in dirs {
                let mut cx = pos.x as i8 + dx;
                let mut cy = pos.y as i8 + dy;
                while cx >= 0 && cx < 10 && cy >= 0 && cy < 10 {
                    let cp = Pos::new(cx as u8, cy as u8);
                    if board.get(cp).is_some() { break; }
                    moves.push(Move::new(pos, cp));
                    cx += dx; cy += dy;
                }
            }
        } else {
            let dy = if piece.color == PieceColor::White { -1 } else { 1 };
            for dx in [1, -1] {
                let nx = pos.x as i8 + dx;
                let ny = pos.y as i8 + dy;
                if nx >= 0 && nx < 10 && ny >= 0 && ny < 10 {
                    let np = Pos::new(nx as u8, ny as u8);
                    if board.get(np).is_none() { moves.push(Move::new(pos, np)); }
                }
            }
        }
        moves
    }
}
