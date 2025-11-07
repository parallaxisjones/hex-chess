use crate::coords::{HexCoord, BoardType};
use crate::pieces::{Piece, PieceType, Color};
use crate::board::Board;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a hexagonal chess variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantConfig {
    pub name: String,
    pub description: String,
    pub board_type: BoardType,
    pub starting_positions: HashMap<HexCoord, Piece>,
    pub pawn_movement: PawnMovement,
    pub special_rules: Vec<SpecialRule>,
}

/// Pawn movement rules (varies by variant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PawnMovement {
    /// Standard pawn movement
    Standard,
    /// Pawns can move in 3 directions
    ThreeDirection,
    /// Custom pawn movement
    Custom(Vec<HexCoord>),
}

/// Special rules for variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecialRule {
    /// En passant rule
    EnPassant,
    /// Castling rule
    Castling,
    /// Custom rule
    Custom(String),
}

impl VariantConfig {
    /// Create a board with the starting positions for this variant
    pub fn create_board(&self) -> Board {
        let mut board = Board::new(self.board_type);
        
        for (&coord, &piece) in &self.starting_positions {
            // Only place pieces on valid coordinates, skip invalid ones
            if let Err(e) = board.place_piece(coord, piece) {
                eprintln!("Warning: Could not place piece at {:?}: {:?}", coord, e);
            }
        }
        
        board
    }
}

/// All available hexagonal chess variants
pub struct Variants;

impl Variants {
    /// Get all available variants
    pub fn all() -> Vec<VariantConfig> {
        vec![
            Self::glinski_chess(),
            Self::mccooey_chess(),
            Self::shafran_chess(),
            Self::brusky_chess(),
            Self::de_vasa_chess(),
            Self::mini_hexchess(),
            Self::glinski_capablanca_chess(),
            Self::mccooey_capablanca_chess(),
        ]
    }

    /// Gliński's Chess - 91 cells, regular hexagon
    pub fn glinski_chess() -> VariantConfig {
        let mut starting_positions = HashMap::new();
        
        // White pieces (bottom of board)
        // Pawns at r=4 (one row forward from back rank) - 9 pawns
        for q in -4..=4 {
            let coord = HexCoord::new(q, 4);
            if coord.in_hexagon(5) {
                starting_positions.insert(coord, Piece::new(PieceType::Pawn, Color::White));
            }
        }
        
        // White back rank (at r=5, within board radius 5) - 6 positions
        // Standard Gliński's setup: Rook, Knight, Bishop, King, Queen, Bishop, Knight, Rook, Rook, Bishop
        // At r=5, valid q range is -5 to 0 (due to hexagon constraint)
        starting_positions.insert(HexCoord::new(-5, 5), Piece::new(PieceType::Rook, Color::White));
        starting_positions.insert(HexCoord::new(-4, 5), Piece::new(PieceType::Knight, Color::White));
        starting_positions.insert(HexCoord::new(-3, 5), Piece::new(PieceType::Bishop, Color::White));
        starting_positions.insert(HexCoord::new(-2, 5), Piece::new(PieceType::Queen, Color::White));
        starting_positions.insert(HexCoord::new(-1, 5), Piece::new(PieceType::King, Color::White));
        starting_positions.insert(HexCoord::new(0, 5), Piece::new(PieceType::Bishop, Color::White));
        
        // Additional White pieces at r=3 to complete the setup
        starting_positions.insert(HexCoord::new(-5, 4), Piece::new(PieceType::Rook, Color::White));
        starting_positions.insert(HexCoord::new(4, 4), Piece::new(PieceType::Knight, Color::White));
        starting_positions.insert(HexCoord::new(-4, 3), Piece::new(PieceType::Rook, Color::White));
        starting_positions.insert(HexCoord::new(5, 4), Piece::new(PieceType::Bishop, Color::White));
        
        // Black pieces (top of board)
        // Pawns at r=-4 (one row forward from back rank) - 9 pawns
        for q in -4..=4 {
            let coord = HexCoord::new(q, -4);
            if coord.in_hexagon(5) {
                starting_positions.insert(coord, Piece::new(PieceType::Pawn, Color::Black));
            }
        }
        
        // Black back rank (at r=-5, within board radius 5) - 6 positions
        // At r=-5, valid q range is 0 to 5 (due to hexagon constraint)
        starting_positions.insert(HexCoord::new(0, -5), Piece::new(PieceType::Bishop, Color::Black));
        starting_positions.insert(HexCoord::new(1, -5), Piece::new(PieceType::King, Color::Black));
        starting_positions.insert(HexCoord::new(2, -5), Piece::new(PieceType::Queen, Color::Black));
        starting_positions.insert(HexCoord::new(3, -5), Piece::new(PieceType::Bishop, Color::Black));
        starting_positions.insert(HexCoord::new(4, -5), Piece::new(PieceType::Knight, Color::Black));
        starting_positions.insert(HexCoord::new(5, -5), Piece::new(PieceType::Rook, Color::Black));
        
        // Additional Black pieces at r=-3 to complete the setup
        starting_positions.insert(HexCoord::new(-5, -4), Piece::new(PieceType::Bishop, Color::Black));
        starting_positions.insert(HexCoord::new(-4, -4), Piece::new(PieceType::Knight, Color::Black));
        starting_positions.insert(HexCoord::new(4, -3), Piece::new(PieceType::Rook, Color::Black));
        starting_positions.insert(HexCoord::new(5, -4), Piece::new(PieceType::Rook, Color::Black));
        
        VariantConfig {
            name: "Gliński's Chess".to_string(),
            description: "91 cells, regular hexagon".to_string(),
            board_type: BoardType::Regular { radius: 5 },
            starting_positions,
            pawn_movement: PawnMovement::Standard,
            special_rules: vec![SpecialRule::EnPassant],
        }
    }

    /// McCooey's Chess - 81 cells, regular hexagon
    pub fn mccooey_chess() -> VariantConfig {
        let mut starting_positions = HashMap::new();
        
        // Similar to Gliński but with 81 cells (radius 3)
        // White pieces
        for q in -3..=3 {
            for r in 2..=3 {
                if HexCoord::new(q, r).in_hexagon(3) {
                    starting_positions.insert(HexCoord::new(q, r), Piece::new(PieceType::Pawn, Color::White));
                }
            }
        }
        
        // White back rank
        starting_positions.insert(HexCoord::new(0, 4), Piece::new(PieceType::King, Color::White));
        starting_positions.insert(HexCoord::new(1, 4), Piece::new(PieceType::Queen, Color::White));
        starting_positions.insert(HexCoord::new(-1, 4), Piece::new(PieceType::Bishop, Color::White));
        starting_positions.insert(HexCoord::new(2, 4), Piece::new(PieceType::Bishop, Color::White));
        starting_positions.insert(HexCoord::new(-2, 4), Piece::new(PieceType::Knight, Color::White));
        starting_positions.insert(HexCoord::new(3, 4), Piece::new(PieceType::Knight, Color::White));
        starting_positions.insert(HexCoord::new(-3, 4), Piece::new(PieceType::Rook, Color::White));
        
        // Black pieces
        for q in -3..=3 {
            for r in -3..=-2 {
                if HexCoord::new(q, r).in_hexagon(3) {
                    starting_positions.insert(HexCoord::new(q, r), Piece::new(PieceType::Pawn, Color::Black));
                }
            }
        }
        
        // Black back rank
        starting_positions.insert(HexCoord::new(0, -4), Piece::new(PieceType::King, Color::Black));
        starting_positions.insert(HexCoord::new(-1, -4), Piece::new(PieceType::Queen, Color::Black));
        starting_positions.insert(HexCoord::new(1, -4), Piece::new(PieceType::Bishop, Color::Black));
        starting_positions.insert(HexCoord::new(-2, -4), Piece::new(PieceType::Bishop, Color::Black));
        starting_positions.insert(HexCoord::new(2, -4), Piece::new(PieceType::Knight, Color::Black));
        starting_positions.insert(HexCoord::new(-3, -4), Piece::new(PieceType::Knight, Color::Black));
        starting_positions.insert(HexCoord::new(3, -4), Piece::new(PieceType::Rook, Color::Black));
        
        VariantConfig {
            name: "McCooey's Chess".to_string(),
            description: "81 cells, regular hexagon".to_string(),
            board_type: BoardType::Regular { radius: 3 },
            starting_positions,
            pawn_movement: PawnMovement::Standard,
            special_rules: vec![SpecialRule::EnPassant],
        }
    }

    /// Shafran's Chess - irregular board
    pub fn shafran_chess() -> VariantConfig {
        // Simplified irregular board for now
        VariantConfig {
            name: "Shafran's Chess".to_string(),
            description: "Irregular board layout".to_string(),
            board_type: BoardType::Irregular,
            starting_positions: HashMap::new(), // TODO: Define irregular layout
            pawn_movement: PawnMovement::Standard,
            special_rules: vec![],
        }
    }

    /// Brusky's Chess - irregular board
    pub fn brusky_chess() -> VariantConfig {
        VariantConfig {
            name: "Brusky's Chess".to_string(),
            description: "Irregular board layout".to_string(),
            board_type: BoardType::Irregular,
            starting_positions: HashMap::new(), // TODO: Define irregular layout
            pawn_movement: PawnMovement::Standard,
            special_rules: vec![],
        }
    }

    /// De Vasa's Chess - irregular board
    pub fn de_vasa_chess() -> VariantConfig {
        VariantConfig {
            name: "De Vasa's Chess".to_string(),
            description: "Irregular board layout".to_string(),
            board_type: BoardType::Irregular,
            starting_positions: HashMap::new(), // TODO: Define irregular layout
            pawn_movement: PawnMovement::Standard,
            special_rules: vec![],
        }
    }

    /// Mini Hexchess - 37 cells
    pub fn mini_hexchess() -> VariantConfig {
        let mut starting_positions = HashMap::new();
        
        // White pieces
        for q in -2..=2 {
            for r in 1..=2 {
                if HexCoord::new(q, r).in_hexagon(2) {
                    starting_positions.insert(HexCoord::new(q, r), Piece::new(PieceType::Pawn, Color::White));
                }
            }
        }
        
        starting_positions.insert(HexCoord::new(0, 3), Piece::new(PieceType::King, Color::White));
        starting_positions.insert(HexCoord::new(1, 3), Piece::new(PieceType::Queen, Color::White));
        starting_positions.insert(HexCoord::new(-1, 3), Piece::new(PieceType::Bishop, Color::White));
        starting_positions.insert(HexCoord::new(2, 3), Piece::new(PieceType::Knight, Color::White));
        starting_positions.insert(HexCoord::new(-2, 3), Piece::new(PieceType::Rook, Color::White));
        
        // Black pieces
        for q in -2..=2 {
            for r in -2..=-1 {
                if HexCoord::new(q, r).in_hexagon(2) {
                    starting_positions.insert(HexCoord::new(q, r), Piece::new(PieceType::Pawn, Color::Black));
                }
            }
        }
        
        starting_positions.insert(HexCoord::new(0, -3), Piece::new(PieceType::King, Color::Black));
        starting_positions.insert(HexCoord::new(-1, -3), Piece::new(PieceType::Queen, Color::Black));
        starting_positions.insert(HexCoord::new(1, -3), Piece::new(PieceType::Bishop, Color::Black));
        starting_positions.insert(HexCoord::new(-2, -3), Piece::new(PieceType::Knight, Color::Black));
        starting_positions.insert(HexCoord::new(2, -3), Piece::new(PieceType::Rook, Color::Black));
        
        VariantConfig {
            name: "Mini Hexchess".to_string(),
            description: "37 cells, small hexagon".to_string(),
            board_type: BoardType::Small,
            starting_positions,
            pawn_movement: PawnMovement::Standard,
            special_rules: vec![],
        }
    }

    /// Gliński-Capablanca Chess - with fairy pieces
    pub fn glinski_capablanca_chess() -> VariantConfig {
        let mut config = Self::glinski_chess();
        config.name = "Gliński-Capablanca Chess".to_string();
        config.description = "91 cells with fairy pieces".to_string();
        
        // Replace some pieces with fairy pieces
        config.starting_positions.insert(HexCoord::new(2, 5), Piece::new(PieceType::Chancellor, Color::White));
        config.starting_positions.insert(HexCoord::new(-2, 5), Piece::new(PieceType::Archbishop, Color::White));
        config.starting_positions.insert(HexCoord::new(-2, -5), Piece::new(PieceType::Chancellor, Color::Black));
        config.starting_positions.insert(HexCoord::new(2, -5), Piece::new(PieceType::Archbishop, Color::Black));
        
        config
    }

    /// McCooey-Capablanca Chess - with fairy pieces
    pub fn mccooey_capablanca_chess() -> VariantConfig {
        let mut config = Self::mccooey_chess();
        config.name = "McCooey-Capablanca Chess".to_string();
        config.description = "81 cells with fairy pieces".to_string();
        
        // Replace some pieces with fairy pieces
        config.starting_positions.insert(HexCoord::new(2, 4), Piece::new(PieceType::Chancellor, Color::White));
        config.starting_positions.insert(HexCoord::new(-2, 4), Piece::new(PieceType::Archbishop, Color::White));
        config.starting_positions.insert(HexCoord::new(-2, -4), Piece::new(PieceType::Chancellor, Color::Black));
        config.starting_positions.insert(HexCoord::new(2, -4), Piece::new(PieceType::Archbishop, Color::Black));
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variant_creation() {
        let glinski = Variants::glinski_chess();
        assert_eq!(glinski.name, "Gliński's Chess");
        assert!(glinski.starting_positions.len() > 0);
    }

    #[test]
    fn test_board_creation_from_variant() {
        let mini = Variants::mini_hexchess();
        let board = mini.create_board();
        assert!(board.pieces.len() > 0);
    }

    #[test]
    fn test_all_variants() {
        let variants = Variants::all();
        assert_eq!(variants.len(), 8);
    }
}
