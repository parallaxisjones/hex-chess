# Latest Changes - November 7, 2025

## Summary
Fixed camera zoom and completed Gliński's Chess starting positions.

## Changes Made

### 1. Camera Zoom - `crates/game/src/lib.rs`
**Location**: `setup` function, line ~130

**Change**: Set initial camera zoom to 1.2 as requested
```rust
projection: OrthographicProjection {
    scale: 1.2, // Default zoom level for comfortable viewing
    ..default()
}
```

**Previous value**: 0.5

### 2. Starting Positions - `crates/core/src/variants.rs`
**Location**: `glinski_chess` function, lines 74-135

**Issue**: Only 20 pieces were being spawned (8 white back rank + 9 white pawns + 3 black pieces)

**Fix**: Completed the full Gliński's Chess starting position with all pieces:

#### White Pieces (19 total):
- **9 Pawns** at r=4 (q from -4 to 4, within hexagon)
- **Back Rank** at r=5 (q from -5 to 0):
  - Rook at (-5, 5)
  - Knight at (-4, 5)
  - Bishop at (-3, 5)
  - Queen at (-2, 5)
  - King at (-1, 5)
  - Bishop at (0, 5)
- **Additional pieces**:
  - Rook at (-5, 4)
  - Knight at (4, 4)
  - Rook at (-4, 3)
  - Bishop at (5, 4)

#### Black Pieces (19 total):
- **9 Pawns** at r=-4 (q from -4 to 4, within hexagon)
- **Back Rank** at r=-5 (q from 0 to 5):
  - Bishop at (0, -5)
  - King at (1, -5)
  - Queen at (2, -5)
  - Bishop at (3, -5)
  - Knight at (4, -5)
  - Rook at (5, -5)
- **Additional pieces**:
  - Bishop at (-5, -4)
  - Knight at (-4, -4)
  - Rook at (4, -3)
  - Rook at (5, -4)

**Total**: 38 pieces (19 per side)

## Build Status
Code changes compile successfully with `cargo check` in the `crates/core` directory.

## Known Issues
- System-level linker issues preventing WASM build (`wasm-ld` not found, `libiconv` not found)
- These are environment issues, not code issues
- The Nix development environment should resolve these, but there may be configuration issues

## Next Steps
1. User needs to fix the build environment (likely use `nix develop` properly)
2. Once built, test the game to verify:
   - Camera starts at zoom level 1.2
   - All 38 pieces are visible on the board
   - Moves work correctly with the new piece positions

