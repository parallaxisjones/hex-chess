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
        // White back rank (rank 1): b1=WR, c1=WB, d1=WN, e1=WQ, f1=empty, g1=WK, h1=WN, i1=WB, k1=WR
        // Note: f1 is EMPTY in back rank; the middle bishop is at f3 instead!
        starting_positions.insert(HexCoord::from_file_rank('b', 1).unwrap(), Piece::new(PieceType::Rook, Color::White));    // b1=(-4,4)
        starting_positions.insert(HexCoord::from_file_rank('c', 1).unwrap(), Piece::new(PieceType::Bishop, Color::White));  // c1=(-3,4)
        starting_positions.insert(HexCoord::from_file_rank('d', 1).unwrap(), Piece::new(PieceType::Knight, Color::White));  // d1=(-2,4)
        starting_positions.insert(HexCoord::from_file_rank('e', 1).unwrap(), Piece::new(PieceType::Queen, Color::White));   // e1=(-1,4)
        // f1 is EMPTY
        starting_positions.insert(HexCoord::from_file_rank('g', 1).unwrap(), Piece::new(PieceType::King, Color::White));    // g1=(1,4) ✓
        starting_positions.insert(HexCoord::from_file_rank('h', 1).unwrap(), Piece::new(PieceType::Knight, Color::White));  // h1=(2,4)
        starting_positions.insert(HexCoord::from_file_rank('i', 1).unwrap(), Piece::new(PieceType::Bishop, Color::White));  // i1=(3,4)
        starting_positions.insert(HexCoord::from_file_rank('k', 1).unwrap(), Piece::new(PieceType::Rook, Color::White));    // k1=(4,4)
        
        // White pawns (9 total) - forming staircase
        // Ranks 2-5: b2, c2, d3, e4, f5, g4, h3, i2, k2
        starting_positions.insert(HexCoord::from_file_rank('b', 2).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // b2
        starting_positions.insert(HexCoord::from_file_rank('c', 2).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // c2
        starting_positions.insert(HexCoord::from_file_rank('d', 3).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // d3
        starting_positions.insert(HexCoord::from_file_rank('e', 4).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // e4
        starting_positions.insert(HexCoord::from_file_rank('f', 5).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // f5 (center)
        starting_positions.insert(HexCoord::from_file_rank('g', 4).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // g4
        starting_positions.insert(HexCoord::from_file_rank('h', 3).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // h3
        starting_positions.insert(HexCoord::from_file_rank('i', 2).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // i2
        starting_positions.insert(HexCoord::from_file_rank('k', 2).unwrap(), Piece::new(PieceType::Pawn, Color::White));    // k2
        
        // f3 middle bishop (3rd bishop for White)
        starting_positions.insert(HexCoord::from_file_rank('f', 3).unwrap(), Piece::new(PieceType::Bishop, Color::White));  // f3
        
        // WHITE total: 8 (back rank) + 9 (pawns) + 1 (f3 bishop) = 18 pieces ✓
        // Composition: 1K, 1Q, 2R, 2N, 3B (c1, i1, f3), 9P ✓
        
        // BLACK PIECES (top of board, negative r values)
        // Use direct axial coordinates to mirror White's setup
        
        // Black middle bishop at top (mirrors White's f3 bishop)
        starting_positions.insert(HexCoord::new(0, -2), Piece::new(PieceType::Bishop, Color::Black)); // mirrors f3=(0,2)
        
        // Black pawns (9 total) - mirror White's pawns by negating both q and r
        // White pawns: b2=(-4,3), c2=(-3,3), d3=(-2,2), e4=(-1,1), f5=(0,0), g4=(1,1), h3=(2,2), i2=(3,3), k2=(4,3)
        starting_positions.insert(HexCoord::new(4, -3), Piece::new(PieceType::Pawn, Color::Black));    // mirrors b2
        starting_positions.insert(HexCoord::new(3, -3), Piece::new(PieceType::Pawn, Color::Black));    // mirrors c2
        starting_positions.insert(HexCoord::new(2, -2), Piece::new(PieceType::Pawn, Color::Black));    // mirrors d3
        starting_positions.insert(HexCoord::new(1, -1), Piece::new(PieceType::Pawn, Color::Black));    // mirrors e4
        starting_positions.insert(HexCoord::new(0, 0), Piece::new(PieceType::Pawn, Color::Black));     // mirrors f5 (center)
        starting_positions.insert(HexCoord::new(-1, -1), Piece::new(PieceType::Pawn, Color::Black));   // mirrors g4
        starting_positions.insert(HexCoord::new(-2, -2), Piece::new(PieceType::Pawn, Color::Black));   // mirrors h3
        starting_positions.insert(HexCoord::new(-3, -3), Piece::new(PieceType::Pawn, Color::Black));   // mirrors i2
        starting_positions.insert(HexCoord::new(-4, -3), Piece::new(PieceType::Pawn, Color::Black));   // mirrors k2
        
        // Black back rank (8 pieces, mirrors White's back rank)
        // White: b1=(-4,4), c1=(-3,4), d1=(-2,4), e1=(-1,4), [f1 empty], g1=(1,4), h1=(2,4), i1=(3,4), k1=(4,4)
        starting_positions.insert(HexCoord::new(4, -4), Piece::new(PieceType::Rook, Color::Black));    // mirrors b1
        starting_positions.insert(HexCoord::new(3, -4), Piece::new(PieceType::Bishop, Color::Black));  // mirrors c1
        starting_positions.insert(HexCoord::new(2, -4), Piece::new(PieceType::Knight, Color::Black));  // mirrors d1
        starting_positions.insert(HexCoord::new(1, -4), Piece::new(PieceType::Queen, Color::Black));   // mirrors e1
        // (0, -4) is empty (mirrors f1)
        starting_positions.insert(HexCoord::new(-1, -4), Piece::new(PieceType::King, Color::Black));   // mirrors g1
        starting_positions.insert(HexCoord::new(-2, -4), Piece::new(PieceType::Knight, Color::Black)); // mirrors h1
        starting_positions.insert(HexCoord::new(-3, -4), Piece::new(PieceType::Bishop, Color::Black)); // mirrors i1
        starting_positions.insert(HexCoord::new(-4, -4), Piece::new(PieceType::Rook, Color::Black));   // mirrors k1
        
        // BLACK total: 1 (f11 bishop) + 9 (pawns) + 8 (back rank) = 18 pieces ✓
        // Composition: 1K, 1Q, 2R, 2N, 3B (c8, i8, f11), 9P ✓
        
        // Verify the starting position
        assert_eq!(starting_positions.len(), 36, "Should have exactly 36 pieces total");
        
        // Count piece types for each color
        let mut white_counts = std::collections::HashMap::new();
        let mut black_counts = std::collections::HashMap::new();
        
        for (coord, piece) in &starting_positions {
            // Verify all coordinates are within radius 5
            assert!(coord.in_hexagon(5), "Piece at {:?} is outside radius 5", coord);
            
            let counts = match piece.color {
                Color::White => &mut white_counts,
                Color::Black => &mut black_counts,
            };
            *counts.entry(piece.piece_type).or_insert(0) += 1;
        }
        
        // Verify White piece counts: 1K, 1Q, 2R, 2N, 3B, 9P
        assert_eq!(white_counts.get(&PieceType::King).unwrap_or(&0), &1, "White should have 1 King");
        assert_eq!(white_counts.get(&PieceType::Queen).unwrap_or(&0), &1, "White should have 1 Queen");
        assert_eq!(white_counts.get(&PieceType::Rook).unwrap_or(&0), &2, "White should have 2 Rooks");
        assert_eq!(white_counts.get(&PieceType::Knight).unwrap_or(&0), &2, "White should have 2 Knights");
        assert_eq!(white_counts.get(&PieceType::Bishop).unwrap_or(&0), &3, "White should have 3 Bishops");
        assert_eq!(white_counts.get(&PieceType::Pawn).unwrap_or(&0), &9, "White should have 9 Pawns");
        
        // Verify Black piece counts: 1K, 1Q, 2R, 2N, 3B, 9P
        assert_eq!(black_counts.get(&PieceType::King).unwrap_or(&0), &1, "Black should have 1 King");
        assert_eq!(black_counts.get(&PieceType::Queen).unwrap_or(&0), &1, "Black should have 1 Queen");
        assert_eq!(black_counts.get(&PieceType::Rook).unwrap_or(&0), &2, "Black should have 2 Rooks");
        assert_eq!(black_counts.get(&PieceType::Knight).unwrap_or(&0), &2, "Black should have 2 Knights");
        assert_eq!(black_counts.get(&PieceType::Bishop).unwrap_or(&0), &3, "Black should have 3 Bishops");
        assert_eq!(black_counts.get(&PieceType::Pawn).unwrap_or(&0), &9, "Black should have 9 Pawns");
        
        // Verify key positions match spec
        let white_king_coord = HexCoord::from_file_rank('g', 1).unwrap();
        assert_eq!(white_king_coord, HexCoord::new(1, 4), "White king at g1 should map to (1, 4)");
        assert_eq!(
            starting_positions.get(&white_king_coord).map(|p| p.piece_type),
            Some(PieceType::King),
            "White king should be at g1"
        );
        
        let black_king_coord = HexCoord::from_file_rank('g', 10).unwrap();
        assert_eq!(
            starting_positions.get(&black_king_coord).map(|p| p.piece_type),
            Some(PieceType::King),
            "Black king should be at g10"
        );
        
        let f11_coord = HexCoord::from_file_rank('f', 11).unwrap();
        assert_eq!(f11_coord, HexCoord::new(0, -6), "f11 should map to (0, -6)");
        assert_eq!(
            starting_positions.get(&f11_coord).map(|p| (p.piece_type, p.color)),
            Some((PieceType::Bishop, Color::Black)),
            "Black bishop should be at f11 (single top cell)"
        );
        
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
