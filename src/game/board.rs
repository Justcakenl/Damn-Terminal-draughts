use crate::game::piece::{Piece, PieceColor, PieceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Pos {
    pub x: u8, // 0-9
    pub y: u8, // 0-9
}

impl Pos {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    pub fn is_valid(&self) -> bool {
        self.x < 10 && self.y < 10
    }

    pub fn is_playable(&self) -> bool {
        // In international draughts, only dark squares are played.
        // Assuming (0,0) is a light square. Playable if x+y is odd.
        (self.x as u8 + self.y as u8) % 2 != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Board {
    pub squares: [[Option<Piece>; 10]; 10],
}

impl Board {
    pub fn new() -> Self {
        let mut squares = [[None; 10]; 10];

        // White pieces (bottom)
        for y in 6..10 {
            for x in 0..10 {
                if (x + y) % 2 != 0 {
                    squares[y as usize][x as usize] = Some(Piece::new(PieceColor::White, PieceType::Man));
                }
            }
        }

        // Black pieces (top)
        for y in 0..4 {
            for x in 0..10 {
                if (x + y) % 2 != 0 {
                    squares[y as usize][x as usize] = Some(Piece::new(PieceColor::Black, PieceType::Man));
                }
            }
        }

        Self { squares }
    }

    pub fn get(&self, pos: Pos) -> Option<&Piece> {
        if pos.is_valid() {
            self.squares[pos.y as usize][pos.x as usize].as_ref()
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: Pos, piece: Option<Piece>) {
        if pos.is_valid() {
            self.squares[pos.y as usize][pos.x as usize] = piece;
        }
    }

    pub fn clear(&mut self, pos: Pos) {
        self.set(pos, None);
    }
}
