# Capture Fix & UI Enhancements - November 7, 2025

## Summary
Fixed piece capture bug, added coordinate labels, and implemented captured pieces display.

## Issues Fixed

### 1. **Capture Bug** ❌ → ✅
**Problem**: When a piece captured another piece, the captured piece entity was never despawned, causing pieces to overlap at the same location.

**Root Cause**: The `handle_hex_click` function only updated the moving piece's coordinate but never removed the captured piece entity from the scene.

**Solution**: Modified `handle_hex_click` to:
1. Check if there's a piece at the destination coordinate before making the move
2. After a successful move, find and despawn the captured piece entity
3. Add the captured piece to the `CapturedPieces` resource for tracking

**Code Changes** (`crates/game/src/lib.rs`, lines ~621-653):
```rust
// Check if there's a piece at the destination to capture
let captured_piece = game_data.game.board.get_piece(coord).copied();

if let Err(e) = game_data.game.make_move(selected, coord) {
    // error handling...
} else {
    // Remove captured piece entity if any
    if let Some(captured) = captured_piece {
        for (entity, chess_piece) in piece_query.iter() {
            if chess_piece.coord == coord && 
               chess_piece.piece.piece_type == captured.piece_type && 
               chess_piece.piece.color == captured.color {
                commands.entity(entity).despawn_recursive();
                captured_pieces.add(captured);
                break;
            }
        }
    }
    
    // Update moving piece coordinate...
}
```

### 2. **Coordinate Labels Added** ✅
**Feature**: Added coordinate labels in (q, r) format around the board perimeter.

**Implementation**: 
- New system `spawn_coordinate_labels` (lines 986-1031)
- Labels are spawned for all perimeter hexes (hexes with at least one invalid neighbor)
- Positioned 30% beyond the hex center (LABEL_DISTANCE = 1.3)
- Font size: 11pt, color: semi-transparent gray (70% opacity)
- Z-index: 5.0 (above pieces)

**Example coordinates shown**: (-5, 5), (0, 5), (5, -5), etc.

### 3. **Captured Pieces Display** ✅
**Feature**: Two UI panels showing captured pieces for each player.

**Layout**:
- **White Lost** (bottom-left): Shows white pieces that were captured by black
- **Black Lost** (top-right): Shows black pieces that were captured by white

**Implementation**:
- New resource `CapturedPieces` with `white` and `black` vectors (lines 85-98)
- New component `CapturedPiecesUI` to tag UI elements by color (lines 118-121)
- System `spawn_captured_pieces_areas` creates the UI panels (lines 1033-1115)
- System `update_captured_pieces_display` updates the display when pieces are captured (lines 1117-1161)

**Display Format**: Compact grid with 3 pieces per row
- Piece symbols: P (Pawn), N (Knight), B (Bishop), R (Rook), Q (Queen), K (King), C (Chancellor), A (Archbishop)
- Example: "P P P\nN B R"

## Files Modified

### `crates/game/src/lib.rs`

**New Resources**:
- `CapturedPieces` (line 85): Tracks captured pieces for both colors

**New Components**:
- `CapturedPiecesUI` (line 118): Tags captured pieces UI text elements
- `CoordinateLabel` (line 123): Tags coordinate label entities

**New Systems**:
- `spawn_coordinate_labels` (line 986): Spawns coordinate labels on board perimeter
- `spawn_captured_pieces_areas` (line 1033): Creates UI panels for captured pieces
- `update_captured_pieces_display` (line 1117): Updates captured pieces display when changed

**Modified Functions**:
- `HexChessPlugin::build` (line 37): Registered new resource and systems
- `setup` (line 128): Added calls to spawn coordinate labels and captured pieces areas
- `handle_input` (line 474): Added `captured_pieces` parameter
- `handle_hex_click` (line 593): Fixed capture logic to despawn captured pieces

## How It Works

### Capture Flow:
1. User clicks on a valid destination hex with their selected piece
2. System checks if there's an enemy piece at the destination
3. Game logic validates and executes the move
4. If a piece was captured:
   - The captured piece entity is found and despawned
   - The piece is added to the `CapturedPieces` resource
   - The UI automatically updates to show the captured piece
5. The moving piece's coordinate is updated

### Coordinate Labels:
- Spawned once at startup
- Only show on perimeter hexes (hexes touching the edge)
- Format: (q, r) where q and r are axial coordinates
- Positioned outside the board for minimal interference with gameplay

### Captured Pieces UI:
- Two separate panels (bottom-left and top-right)
- Each panel shows pieces lost by that color
- Automatically updates using Bevy's change detection
- Compact grid format for space efficiency

## Testing Recommendations

1. **Capture Test**: 
   - Move a piece to capture an enemy piece
   - Verify the captured piece disappears from the board
   - Verify the captured piece appears in the appropriate UI panel

2. **Multiple Captures Test**:
   - Capture several pieces
   - Verify they all appear in the captured pieces display
   - Verify the grid format (3 per row)

3. **Coordinate Labels Test**:
   - Zoom in/out to verify labels remain visible
   - Pan around the board to verify all perimeter hexes have labels
   - Verify labels are in (q, r) format

## Known Issues

- Build environment requires Nix development shell (system-level linker issues)
- Use `nix develop` then `cd crates/game && trunk serve --release` to build and test

## Next Steps

Once the game is rebuilt and loaded:
1. Test piece captures to ensure no more overlapping pieces
2. Verify coordinate labels are visible around the board
3. Verify captured pieces display in the UI panels
4. Play a few moves to ensure everything works smoothly

