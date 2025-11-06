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
    pub fn to_pixel(self) -> (f32, f32) {
        let x = 3.0_f32.sqrt() / 2.0 * self.q as f32 + 3.0_f32.sqrt() / 4.0 * self.r as f32;
        let y = 3.0 / 4.0 * self.r as f32;
        (x, y)
    }

    /// Convert from pixel coordinates to hex coordinates
    pub fn from_pixel(x: f32, y: f32) -> Self {
        let q = (2.0 / 3.0_f32.sqrt() * x - 1.0 / 3.0 * y).round() as i32;
        let r = (2.0 / 3.0 * y).round() as i32;
        Self::new(q, r)
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
