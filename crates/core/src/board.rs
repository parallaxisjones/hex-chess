use crate::coords::{HexCoord, BoardType};
use crate::pieces::{Piece, PieceType, Color};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a hexagonal chess board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// The type of board (regular, irregular, small)
    pub board_type: BoardType,
    /// Map of coordinates to pieces
    pub pieces: HashMap<HexCoord, Piece>,
    /// Valid coordinates for this board
    pub valid_coords: std::collections::HashSet<HexCoord>,
    /// Cell colors for rendering (3 colors for regular hex boards)
    pub cell_colors: HashMap<HexCoord, CellColor>,
}

/// Cell colors for hexagonal boards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellColor {
    Light,
    Medium,
    Dark,
}

impl Board {
    /// Create a new empty board
    pub fn new(board_type: BoardType) -> Self {
        let valid_coords = board_type.valid_coords();
        let cell_colors = Self::generate_cell_colors(&valid_coords, board_type);
        
        Self {
            board_type,
            pieces: HashMap::new(),
            valid_coords,
            cell_colors,
        }
    }

    /// Generate cell colors for the board
    fn generate_cell_colors(
        coords: &std::collections::HashSet<HexCoord>,
        board_type: BoardType,
    ) -> HashMap<HexCoord, CellColor> {
        let mut colors = HashMap::new();
        
        for &coord in coords {
            let color = match board_type {
                BoardType::Regular { .. } | BoardType::Small => {
                    // Regular hex boards use 3 colors in a pattern
                    let (q, r, s) = coord.to_cube();
                    match (q + r + s) % 3 {
                        0 => CellColor::Light,
                        1 => CellColor::Medium,
                        _ => CellColor::Dark,
                    }
                }
                BoardType::Irregular => {
                    // Irregular boards will have custom color schemes
                    CellColor::Light
                }
            };
            colors.insert(coord, color);
        }
        
        colors
    }

    /// Place a piece on the board
    pub fn place_piece(&mut self, coord: HexCoord, piece: Piece) -> Result<(), BoardError> {
        if !self.valid_coords.contains(&coord) {
            return Err(BoardError::InvalidCoordinate);
        }
        self.pieces.insert(coord, piece);
        Ok(())
    }

    /// Remove a piece from the board
    pub fn remove_piece(&mut self, coord: HexCoord) -> Option<Piece> {
        self.pieces.remove(&coord)
    }

    /// Get a piece at a coordinate
    pub fn get_piece(&self, coord: HexCoord) -> Option<&Piece> {
        self.pieces.get(&coord)
    }

    /// Check if a coordinate is occupied
    pub fn is_occupied(&self, coord: HexCoord) -> bool {
        self.pieces.contains_key(&coord)
    }

    /// Check if a coordinate is valid for this board
    pub fn is_valid_coord(&self, coord: HexCoord) -> bool {
        self.valid_coords.contains(&coord)
    }

    /// Get all pieces of a specific color
    pub fn get_pieces_by_color(&self, color: Color) -> Vec<(HexCoord, &Piece)> {
        self.pieces
            .iter()
            .filter(|(_, piece)| piece.color == color)
            .map(|(coord, piece)| (*coord, piece))
            .collect()
    }

    /// Get the king of a specific color
    pub fn get_king(&self, color: Color) -> Option<HexCoord> {
        self.pieces
            .iter()
            .find(|(_, piece)| piece.color == color && piece.piece_type == PieceType::King)
            .map(|(coord, _)| *coord)
    }

    /// Move a piece from one coordinate to another
    pub fn move_piece(&mut self, from: HexCoord, to: HexCoord) -> Result<Piece, BoardError> {
        if !self.is_valid_coord(from) || !self.is_valid_coord(to) {
            return Err(BoardError::InvalidCoordinate);
        }
        
        let piece = self.pieces.remove(&from)
            .ok_or(BoardError::NoPieceAtCoordinate)?;
        
        // If there's a piece at the destination, it's captured
        let captured = self.pieces.insert(to, piece);
        
        Ok(captured.unwrap_or_else(|| Piece {
            piece_type: PieceType::Pawn, // Dummy piece for captures
            color: Color::White,
        }))
    }

    /// Get all valid moves for a piece at a coordinate
    pub fn get_valid_moves(&self, coord: HexCoord) -> Vec<HexCoord> {
        let piece = match self.get_piece(coord) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut moves = Vec::new();
        
        // Get all possible moves for this piece type
        let possible_moves = piece.piece_type.get_moves(coord, self);
        
        for target in possible_moves {
            // Check if the move is valid (not blocked, doesn't put own king in check, etc.)
            if self.is_valid_move(coord, target) {
                moves.push(target);
            }
        }
        
        moves
    }

    /// Check if a move is valid (basic validation, not considering check)
    fn is_valid_move(&self, from: HexCoord, to: HexCoord) -> bool {
        // Can't move to invalid coordinates
        if !self.is_valid_coord(to) {
            return false;
        }
        
        // Can't capture own piece
        if let Some(target_piece) = self.get_piece(to) {
            if let Some(from_piece) = self.get_piece(from) {
                if target_piece.color == from_piece.color {
                    return false;
                }
            }
        }
        
        // TODO: Add path checking for sliding pieces
        // TODO: Add check validation
        
        true
    }

    /// Create a copy of the board with a move applied
    pub fn with_move(&self, from: HexCoord, to: HexCoord) -> Result<Self, BoardError> {
        let mut new_board = self.clone();
        new_board.move_piece(from, to)?;
        Ok(new_board)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BoardError {
    #[error("Invalid coordinate for this board")]
    InvalidCoordinate,
    #[error("No piece at the specified coordinate")]
    NoPieceAtCoordinate,
    #[error("Invalid move")]
    InvalidMove,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pieces::{Piece, PieceType, Color};

    #[test]
    fn test_board_creation() {
        let board = Board::new(BoardType::Regular { radius: 2 });
        assert_eq!(board.pieces.len(), 0);
        assert!(board.valid_coords.len() > 0);
    }

    #[test]
    fn test_piece_placement() {
        let mut board = Board::new(BoardType::Regular { radius: 1 });
        let piece = Piece {
            piece_type: PieceType::King,
            color: Color::White,
        };
        
        let coord = HexCoord::new(0, 0);
        assert!(board.place_piece(coord, piece.clone()).is_ok());
        assert_eq!(board.get_piece(coord), Some(&piece));
    }

    #[test]
    fn test_invalid_coordinate() {
        let mut board = Board::new(BoardType::Regular { radius: 1 });
        let piece = Piece {
            piece_type: PieceType::King,
            color: Color::White,
        };
        
        let invalid_coord = HexCoord::new(10, 10);
        assert!(board.place_piece(invalid_coord, piece).is_err());
    }
}
