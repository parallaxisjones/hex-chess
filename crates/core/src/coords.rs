use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Axial coordinates for hexagonal grids
/// q = column, r = row in hex coordinate system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert to cube coordinates (q, r, s) where s = -q - r
    pub fn to_cube(self) -> (i32, i32, i32) {
        (self.q, self.r, -self.q - self.r)
    }

    /// Create from cube coordinates
    pub fn from_cube(q: i32, r: i32, _s: i32) -> Self {
        Self { q, r }
    }

    /// Get the 6 neighboring hex coordinates
    pub fn neighbors(self) -> [HexCoord; 6] {
        [
            HexCoord::new(self.q + 1, self.r),     // East
            HexCoord::new(self.q + 1, self.r - 1), // Northeast
            HexCoord::new(self.q, self.r - 1),     // Northwest
            HexCoord::new(self.q - 1, self.r),     // West
            HexCoord::new(self.q - 1, self.r + 1), // Southwest
            HexCoord::new(self.q, self.r + 1),     // Southeast
        ]
    }

    /// Get the 6 diagonal neighbors (across corners)
    pub fn diagonal_neighbors(self) -> [HexCoord; 6] {
        [
            HexCoord::new(self.q + 2, self.r - 1), // Northeast diagonal
            HexCoord::new(self.q + 1, self.r - 2), // Northwest diagonal
            HexCoord::new(self.q - 1, self.r - 1), // West diagonal
            HexCoord::new(self.q - 2, self.r + 1), // Southwest diagonal
            HexCoord::new(self.q - 1, self.r + 2), // Southeast diagonal
            HexCoord::new(self.q + 1, self.r + 1), // East diagonal
        ]
    }

    /// Calculate distance to another hex coordinate
    pub fn distance_to(self, other: HexCoord) -> i32 {
        let (q1, r1, s1) = self.to_cube();
        let (q2, r2, s2) = other.to_cube();
        (q1 - q2).abs() + (r1 - r2).abs() + (s1 - s2).abs()
    }

    /// Get all coordinates in a line from this point to another
    pub fn line_to(self, other: HexCoord) -> Vec<HexCoord> {
        let distance = self.distance_to(other);
        if distance == 0 {
            return vec![self];
        }

        let mut result = Vec::new();
        for i in 0..=distance {
            let q = self.q + (other.q - self.q) * i / distance;
            let r = self.r + (other.r - self.r) * i / distance;
            result.push(HexCoord::new(q, r));
        }
        result
    }

    /// Check if this coordinate is within a regular hexagon of given radius
    pub fn in_hexagon(self, radius: i32) -> bool {
        let (q, r, s) = self.to_cube();
        q.abs() <= radius && r.abs() <= radius && s.abs() <= radius
    }

    /// Convert to pixel coordinates for rendering
    /// Assumes hex size of 1.0
    /// Uses flat-top hexagon orientation (flat edges on left/right, points on top/bottom)
    pub fn to_pixel(self) -> (f32, f32) {
        let x = 3.0 / 4.0 * self.q as f32;
        let y = 3.0_f32.sqrt() / 2.0 * self.r as f32 + 3.0_f32.sqrt() / 4.0 * self.q as f32;
        (x, -y)  // Negate y so negative r is at top of screen
    }

    /// Convert from pixel coordinates to hex coordinates
    /// Uses flat-top hexagon orientation
    pub fn from_pixel(x: f32, y: f32) -> Self {
        let y = -y;  // Invert y coordinate
        let q = (2.0 / 3.0 * x).round() as i32;
        let r = (2.0 / 3.0_f32.sqrt() * y - 1.0 / 3.0 * x).round() as i32;
        Self::new(q, r)
    }
    
    /// Convert Gliński file/rank notation to axial coordinates
    /// Files: a b c d e f g h i k l (no j)
    /// Ranks: 1-11 (White starts at 1-5, Black at 7-11)
    /// Returns None if invalid file/rank combination
    pub fn from_file_rank(file: char, rank: u8) -> Option<Self> {
        file_rank_to_axial(file, rank)
    }
}

/// Convert Gliński file/rank notation to axial (q, r) coordinates
/// Based on authoritative mapping for radius-5 flat-top hexagonal board
/// Files: a b c d e f g h i k l (no j), where f is the vertical spine at q=0
/// Ranks: 1-11, with White at bottom (ranks 1-6) and Black at top (ranks 7-11)
pub fn file_rank_to_axial(file: char, rank: u8) -> Option<HexCoord> {
    // Map file character to q offset
    let q = match file {
        'a' => -5,
        'b' => -4,
        'c' => -3,
        'd' => -2,
        'e' => -1,
        'f' => 0,
        'g' => 1,
        'h' => 2,
        'i' => 3,
        'k' => 4,
        'l' => 5,
        _ => return None,  // Invalid file (including 'j')
    };
    
    // Map rank to r coordinate
    // White starts at bottom (positive r), Black at top (negative r)
    let r = match rank {
        1 => 4,    // Rank 1: 11 cells (a-l)
        2 => 3,    // Rank 2: 11 cells (a-l)
        3 => 2,    // Rank 3: 11 cells (a-l)
        4 => 1,    // Rank 4: 11 cells (a-l)
        5 => 0,    // Rank 5: 11 cells (a-l)
        6 => -1,   // Rank 6: 11 cells (a-l)
        7 => -2,   // Rank 7: 9 cells (b-k, no a or l)
        8 => -3,   // Rank 8: 7 cells (c-i)
        9 => -4,   // Rank 9: 5 cells (d-h)
        10 => -5,  // Rank 10: 3 cells (e-g)
        11 => -6,  // Rank 11: 1 cell (f only)
        _ => return None,  // Invalid rank
    };
    
    // Validate that the file/rank combination is valid for the given rank
    let valid = match rank {
        1..=6 => true,  // All files a-l valid for ranks 1-6
        7 => file != 'a' && file != 'l',  // Rank 7: b-k only
        8 => q >= -3 && q <= 3,  // Rank 8: c-i (q: -3 to 3)
        9 => q >= -2 && q <= 2,  // Rank 9: d-h (q: -2 to 2)
        10 => q >= -1 && q <= 1,  // Rank 10: e-g (q: -1 to 1)
        11 => q == 0,  // Rank 11: f only (q: 0)
        _ => false,
    };
    
    if valid {
        Some(HexCoord::new(q, r))
    } else {
        None
    }
}

impl std::ops::Add for HexCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl std::ops::Sub for HexCoord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.q - rhs.q, self.r - rhs.r)
    }
}

/// Hexagonal board types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardType {
    /// Regular hexagon with radius (number of rings)
    Regular { radius: i32 },
    /// Irregular board with custom shape
    Irregular,
    /// Small hexagon (37 cells)
    Small,
}

impl BoardType {
    /// Get all valid coordinates for this board type
    pub fn valid_coords(self) -> HashSet<HexCoord> {
        match self {
            BoardType::Regular { radius } => {
                let mut coords = HashSet::new();
                for q in -radius..=radius {
                    for r in -radius..=radius {
                        let coord = HexCoord::new(q, r);
                        if coord.in_hexagon(radius) {
                            coords.insert(coord);
                        }
                    }
                }
                coords
            }
            BoardType::Small => {
                // Mini Hexchess has 37 cells in a specific pattern
                let mut coords = HashSet::new();
                for q in -3..=3 {
                    for r in -3..=3 {
                        let coord = HexCoord::new(q, r);
                        if coord.in_hexagon(3) {
                            coords.insert(coord);
                        }
                    }
                }
                coords
            }
            BoardType::Irregular => {
                // Will be defined per variant
                HashSet::new()
            }
        }
    }

    /// Get the center coordinate of the board
    pub fn center(self) -> HexCoord {
        match self {
            BoardType::Regular { .. } | BoardType::Small => HexCoord::new(0, 0),
            BoardType::Irregular => HexCoord::new(0, 0), // Will be overridden per variant
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_coord_operations() {
        let coord1 = HexCoord::new(1, 2);
        let coord2 = HexCoord::new(3, 1);
        
        assert_eq!(coord1 + coord2, HexCoord::new(4, 3));
        assert_eq!(coord2 - coord1, HexCoord::new(2, -1));
    }

    #[test]
    fn test_distance_calculation() {
        let center = HexCoord::new(0, 0);
        let neighbor = HexCoord::new(1, 0);
        let far = HexCoord::new(2, 1);
        
        assert_eq!(center.distance_to(neighbor), 1);
        assert_eq!(center.distance_to(far), 2);
    }

    #[test]
    fn test_hexagon_bounds() {
        let center = HexCoord::new(0, 0);
        let edge = HexCoord::new(2, 0);
        let outside = HexCoord::new(3, 0);
        
        assert!(center.in_hexagon(2));
        assert!(edge.in_hexagon(2));
        assert!(!outside.in_hexagon(2));
    }

    #[test]
    fn test_regular_board_coords() {
        let board = BoardType::Regular { radius: 1 };
        let coords = board.valid_coords();
        
        // Should have 7 cells (center + 6 neighbors)
        assert_eq!(coords.len(), 7);
        assert!(coords.contains(&HexCoord::new(0, 0)));
        assert!(coords.contains(&HexCoord::new(1, 0)));
        assert!(coords.contains(&HexCoord::new(-1, 0)));
    }
}
