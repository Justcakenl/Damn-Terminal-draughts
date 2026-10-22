#[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    pub fn opposite(&self) -> Self {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PieceType {
    Man,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Piece {
    pub color: PieceColor,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn new(color: PieceColor, piece_type: PieceType) -> Self {
        Self { color, piece_type }
    }

    pub fn is_king(&self) -> bool {
        self.piece_type == PieceType::King
    }
}

impl std::fmt::Debug for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceColor::White => write!(f, "White"),
            PieceColor::Black => write!(f, "Black"),
        }
    }
}
