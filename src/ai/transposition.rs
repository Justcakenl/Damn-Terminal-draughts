use std::collections::HashMap;
use crate::game::board::Board;

#[derive(Default)]
pub struct TranspositionTable { entries: HashMap<u64, (u8, i32)> }
#[allow(dead_code)]
impl TranspositionTable {
    pub fn new() -> Self { Self::default() }
    pub fn get(&self, key: u64, depth: u8) -> Option<i32> { self.entries.get(&key).filter(|(d, _)| *d >= depth).map(|(_, v)| *v) }
    pub fn insert(&mut self, key: u64, depth: u8, score: i32) { self.entries.insert(key, (depth, score)); }
}
#[allow(dead_code)]
pub fn hash(board: &Board) -> u64 {
    let mut h = 1469598103934665603u64;
    for row in board.squares { for piece in row { h ^= piece.map_or(0, |p| (p.color as u64 + 1) * 3 + p.piece_type as u64); h = h.wrapping_mul(1099511628211); } }
    h
}
