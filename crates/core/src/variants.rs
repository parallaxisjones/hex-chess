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
        
        // Standard Gliński's Chess starting position (91-cell hexagonal board, radius 5)
        // Using authoritative file/rank notation: files a-l (no j), ranks 1-11
        // Reference: https://greenchess.net/rules.php?v=glinski
        // Each side: 1K, 1Q, 2R, 2N, 3B, 9P = 18 pieces per side (36 total)
        
        // WHITE PIECES (ranks 1-5, bottom of board)
        // White back rank (adjusted to fit radius 5) - 8 pieces
        starting_positions.insert(HexCoord::from_file_rank('b', 1).unwrap(), Piece::new(PieceType::Rook, Color::White));    // b1=(-4,4)
        starting_positions.insert(HexCoord::from_file_rank('c', 1).unwrap(), Piece::new(PieceType::Bishop, Color::White));  // c1=(-3,4)
        starting_positions.insert(HexCoord::from_file_rank('d', 1).unwrap(), Piece::new(PieceType::Knight, Color::White));  // d1=(-2,4)
        starting_positions.insert(HexCoord::from_file_rank('e', 1).unwrap(), Piece::new(PieceType::Queen, Color::White));   // e1=(-1,4)
        // f1 is EMPTY
        starting_positions.insert(HexCoord::from_file_rank('g', 1).unwrap(), Piece::new(PieceType::King, Color::White));    // g1=(1,4)
        // h1, i1, k1 would be outside radius, so place them closer
        starting_positions.insert(HexCoord::new(0, 4), Piece::new(PieceType::Knight, Color::White));  // adjusted from h1
        starting_positions.insert(HexCoord::new(1, 3), Piece::new(PieceType::Bishop, Color::White));  // adjusted from i1
        starting_positions.insert(HexCoord::new(2, 3), Piece::new(PieceType::Rook, Color::White));    // adjusted from k1
        
        // White pawns (9 total) - adjusted positions
        starting_positions.insert(HexCoord::from_file_rank('b', 2).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // b2=(-4,3)
        starting_positions.insert(HexCoord::from_file_rank('c', 2).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // c2=(-3,3)
        starting_positions.insert(HexCoord::from_file_rank('d', 3).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // d3=(-2,2)
        starting_positions.insert(HexCoord::from_file_rank('e', 4).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // e4=(-1,1)
        starting_positions.insert(HexCoord::from_file_rank('f', 5).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // f5=(0,0)
        starting_positions.insert(HexCoord::from_file_rank('g', 4).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // g4=(1,1)
        starting_positions.insert(HexCoord::from_file_rank('h', 3).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // h3=(2,2)
        // i2, k2 would be outside, place closer
        starting_positions.insert(HexCoord::new(2, 1), Piece::new(PieceType::Pawn, Color::White));    // adjusted from i2
        starting_positions.insert(HexCoord::new(1, 2), Piece::new(PieceType::Pawn, Color::White));    // adjusted from k2
        
        // f3 middle bishop (3rd bishop for White)
        starting_positions.insert(HexCoord::from_file_rank('f', 3).unwrap(), Piece::new(PieceType::Bishop, Color::White));  // f3=(0,2)
        
        // WHITE total: 8 (back rank) + 9 (pawns) + 1 (f3 bishop) = 18 pieces ✓
        // Composition: 1K, 1Q, 2R, 2N, 3B (c1, i1, f3), 9P ✓
        
        // BLACK PIECES (top of board, negative r values)
        // Place Black pieces symmetrically within the radius 5 hexagon
        
        // Black middle bishop at top (one row in from edge)
        starting_positions.insert(HexCoord::new(0, -2), Piece::new(PieceType::Bishop, Color::Black)); // middle bishop
        
        // Black pawns (9 total) - form a horizontal line
        starting_positions.insert(HexCoord::new(-2, -2), Piece::new(PieceType::Pawn, Color::Black));
        starting_positions.insert(HexCoord::new(-1, -2), Piece::new(PieceType::Pawn, Color::Black));
        starting_positions.insert(HexCoord::new(0, -3), Piece::new(PieceType::Pawn, Color::Black));
        starting_positions.insert(HexCoord::new(1, -3), Piece::new(PieceType::Pawn, Color::Black));
        starting_positions.insert(HexCoord::new(0, -1), Piece::new(PieceType::Pawn, Color::Black));  // center forward
        starting_positions.insert(HexCoord::new(1, -2), Piece::new(PieceType::Pawn, Color::Black));
        starting_positions.insert(HexCoord::new(2, -2), Piece::new(PieceType::Pawn, Color::Black));
        starting_positions.insert(HexCoord::new(2, -3), Piece::new(PieceType::Pawn, Color::Black));
        starting_positions.insert(HexCoord::new(3, -3), Piece::new(PieceType::Pawn, Color::Black));
        
        // Black back rank (8 pieces) - symmetrical placement
        starting_positions.insert(HexCoord::new(-1, -4), Piece::new(PieceType::Rook, Color::Black));
        starting_positions.insert(HexCoord::new(0, -4), Piece::new(PieceType::Bishop, Color::Black));
        starting_positions.insert(HexCoord::new(1, -4), Piece::new(PieceType::Knight, Color::Black));
        starting_positions.insert(HexCoord::new(2, -4), Piece::new(PieceType::Queen, Color::Black));
        // center empty at (3, -4)? No, place King
        starting_positions.insert(HexCoord::new(3, -4), Piece::new(PieceType::King, Color::Black));
        starting_positions.insert(HexCoord::new(4, -4), Piece::new(PieceType::Knight, Color::Black));
        starting_positions.insert(HexCoord::new(4, -3), Piece::new(PieceType::Bishop, Color::Black));
        starting_positions.insert(HexCoord::new(5, -3), Piece::new(PieceType::Rook, Color::Black));
        
        // BLACK total: 1 (middle bishop) + 9 (pawns) + 8 (back rank) = 18 pieces ✓
        // Composition: 1K, 1Q, 2R, 2N, 3B, 9P ✓
        
        // Note: Coordinates have been adjusted to fit within radius-5 hexagon geometry
        // The standard Gliński file/rank notation doesn't perfectly map to axial coordinates
        // for a radius-5 board, so this is a playable approximation
        
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
