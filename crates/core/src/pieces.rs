use crate::coords::HexCoord;
use crate::board::Board;
use serde::{Deserialize, Serialize};

/// Chess piece types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    // Fairy pieces for Capablanca variants
    Chancellor,  // Rook + Knight
    Archbishop,  // Bishop + Knight
}

/// Piece colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

/// A chess piece
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self { piece_type, color }
    }

    /// Get the symbol for this piece (for display)
    pub fn symbol(&self) -> char {
        let base_symbol = match self.piece_type {
            PieceType::King => 'K',
            PieceType::Queen => 'Q',
            PieceType::Rook => 'R',
            PieceType::Bishop => 'B',
            PieceType::Knight => 'N',
            PieceType::Pawn => 'P',
            PieceType::Chancellor => 'C',
            PieceType::Archbishop => 'A',
        };

        match self.color {
            Color::White => base_symbol,
            Color::Black => base_symbol.to_ascii_lowercase(),
        }
    }
}

impl PieceType {
    /// Get all possible moves for this piece type from a given position
    pub fn get_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        match self {
            PieceType::King => self.king_moves(from, board),
            PieceType::Queen => self.queen_moves(from, board),
            PieceType::Rook => self.rook_moves(from, board),
            PieceType::Bishop => self.bishop_moves(from, board),
            PieceType::Knight => self.knight_moves(from, board),
            PieceType::Pawn => self.pawn_moves(from, board),
            PieceType::Chancellor => self.chancellor_moves(from, board),
            PieceType::Archbishop => self.archbishop_moves(from, board),
        }
    }

    /// King moves: one step in any of the 6 directions (like a rook, but only one step)
    /// In Gliński's Chess, the king moves to the 6 adjacent hexes, not diagonals
    fn king_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        let mut moves = Vec::new();
        
        // All 6 adjacent hexes (rook-like movement, but only one step)
        for neighbor in from.neighbors() {
            if board.is_valid_coord(neighbor) {
                moves.push(neighbor);
            }
        }
        
        moves
    }

    /// Queen moves: combination of rook and bishop
    fn queen_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        let mut moves = Vec::new();
        moves.extend(self.rook_moves(from, board));
        moves.extend(self.bishop_moves(from, board));
        moves
    }

    /// Rook moves: straight lines in 6 directions
    fn rook_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        let mut moves = Vec::new();
        
        // 6 directions for hexagonal rook
        let directions = [
            HexCoord::new(1, 0),      // East
            HexCoord::new(1, -1),     // Northeast
            HexCoord::new(0, -1),     // Northwest
            HexCoord::new(-1, 0),     // West
            HexCoord::new(-1, 1),     // Southwest
            HexCoord::new(0, 1),      // Southeast
        ];
        
        for direction in directions {
            let mut current = from + direction;
            while board.is_valid_coord(current) {
                moves.push(current);
                if board.is_occupied(current) {
                    break; // Can't move through pieces
                }
                current = current + direction;
            }
        }
        
        moves
    }

    /// Bishop moves: diagonal lines in 6 directions
    fn bishop_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        let mut moves = Vec::new();
        
        // 6 diagonal directions for hexagonal bishop
        let directions = [
            HexCoord::new(2, -1),     // Northeast diagonal
            HexCoord::new(1, -2),     // Northwest diagonal
            HexCoord::new(-1, -1),    // West diagonal
            HexCoord::new(-2, 1),     // Southwest diagonal
            HexCoord::new(-1, 2),     // Southeast diagonal
            HexCoord::new(1, 1),      // East diagonal
        ];
        
        for direction in directions {
            let mut current = from + direction;
            while board.is_valid_coord(current) {
                moves.push(current);
                if board.is_occupied(current) {
                    break; // Can't move through pieces
                }
                current = current + direction;
            }
        }
        
        moves
    }

    /// Knight moves: L-shaped moves adapted for hex geometry
    fn knight_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        let mut moves = Vec::new();
        
        // Hexagonal knight moves (L-shaped in hex coordinates)
        let knight_moves = [
            HexCoord::new(2, -1),     // 2 east, 1 northwest
            HexCoord::new(1, -2),     // 1 east, 2 northwest
            HexCoord::new(-1, -1),    // 1 west, 1 northwest
            HexCoord::new(-2, 1),     // 2 west, 1 southeast
            HexCoord::new(-1, 2),     // 1 west, 2 southeast
            HexCoord::new(1, 1),      // 1 east, 1 southeast
            HexCoord::new(3, -2),     // 3 east, 2 northwest
            HexCoord::new(2, -3),     // 2 east, 3 northwest
            HexCoord::new(-2, -1),    // 2 west, 1 northwest
            HexCoord::new(-3, 2),     // 3 west, 2 southeast
            HexCoord::new(-2, 3),     // 2 west, 3 southeast
            HexCoord::new(2, 1),      // 2 east, 1 southeast
        ];
        
        for knight_move in knight_moves {
            let target = from + knight_move;
            if board.is_valid_coord(target) {
                moves.push(target);
            }
        }
        
        moves
    }

    /// Pawn moves: Gliński's Chess rules
    /// Pawns move forward to the adjacent cell directly ahead (1 direction)
    /// Pawns capture diagonally forward to the sides (2 directions)
    fn pawn_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        let mut moves = Vec::new();
        
        let piece = board.get_piece(from).unwrap();
        
        // In Gliński's Chess, pawns move straight forward (1 direction)
        let forward_direction = match piece.color {
            Color::White => HexCoord::new(0, -1),   // Straight forward (northwest)
            Color::Black => HexCoord::new(0, 1),    // Straight forward (southeast)
        };
        
        // Pawns can move forward to an empty square
        let forward_target = from + forward_direction;
        if board.is_valid_coord(forward_target) && !board.is_occupied(forward_target) {
            moves.push(forward_target);
        }
        
        // Pawns capture diagonally forward (2 directions)
        let capture_directions = match piece.color {
            Color::White => [
                HexCoord::new(-1, -1), // Forward-left diagonal (west)
                HexCoord::new(1, -1),  // Forward-right diagonal (northeast)
            ],
            Color::Black => [
                HexCoord::new(-1, 1), // Forward-left diagonal (southwest)
                HexCoord::new(1, 1),  // Forward-right diagonal (east)
            ],
        };
        
        for capture_dir in capture_directions {
            let capture_target = from + capture_dir;
            if board.is_valid_coord(capture_target) {
                if let Some(target_piece) = board.get_piece(capture_target) {
                    // Can capture enemy pieces diagonally forward
                    if target_piece.color != piece.color {
                        moves.push(capture_target);
                    }
                }
            }
        }
        
        moves
    }

    /// Chancellor moves: combination of rook and knight
    fn chancellor_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        let mut moves = Vec::new();
        moves.extend(self.rook_moves(from, board));
        moves.extend(self.knight_moves(from, board));
        moves
    }

    /// Archbishop moves: combination of bishop and knight
    fn archbishop_moves(&self, from: HexCoord, board: &Board) -> Vec<HexCoord> {
        let mut moves = Vec::new();
        moves.extend(self.bishop_moves(from, board));
        moves.extend(self.knight_moves(from, board));
        moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn test_king_moves() {
        let board = Board::new(BoardType::Regular { radius: 2 });
        let king = PieceType::King;
        let center = HexCoord::new(0, 0);
        
        let moves = king.get_moves(center, &board);
        // King should have 12 possible moves (6 neighbors + 6 diagonals)
        assert_eq!(moves.len(), 12);
    }

    #[test]
    fn test_rook_moves() {
        let board = Board::new(BoardType::Regular { radius: 2 });
        let rook = PieceType::Rook;
        let center = HexCoord::new(0, 0);
        
        let moves = rook.get_moves(center, &board);
        // Rook should be able to move in 6 directions
        assert!(moves.len() > 6);
    }

    #[test]
    fn test_piece_symbols() {
        let white_king = Piece::new(PieceType::King, Color::White);
        let black_king = Piece::new(PieceType::King, Color::Black);
        
        assert_eq!(white_king.symbol(), 'K');
        assert_eq!(black_king.symbol(), 'k');
    }
}
