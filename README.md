# Hexagonal Chess WASM Game

A Rust/Bevy-based hexagonal chess game supporting all variants from [greenchess.net](https://greenchess.net/variants.php?cat=6) with WebRTC p2p multiplayer.

## Features

- **8 Hex Chess Variants**: Gliński's, McCooey's, Shafran's, Brusky's, De Vasa's, Mini Hexchess, and Capablanca variants
- **WebRTC P2P Multiplayer**: True peer-to-peer gameplay with minimal server infrastructure
- **WASM Build**: Runs entirely in the browser with near-native performance
- **NixOS Deployment**: Complete dev environment and production deployment with Nix

## Architecture

```
hex-chess/
├── crates/
│   ├── core/          # Pure game logic (no rendering)
│   ├── game/          # Bevy WASM app
│   └── signaling/     # Minimal WebRTC signaling server
├── web/               # Static web assets
└── deploy/nixos/      # NixOS configuration
```

## Quick Start

### Development

1. **Enter the Nix development shell**:
   ```bash
   nix develop
   ```

2. **Build the WASM game**:
   ```bash
   cd crates/game
   trunk build --release
   ```

3. **Start the signaling server** (in another terminal):
   ```bash
   cd crates/signaling
   cargo run
   ```

4. **Serve the web app**:
   ```bash
   cd web
   python -m http.server 8080
   ```

5. **Open your browser** to `http://localhost:8080`

### Production Deployment

1. **Build all packages**:
   ```bash
   nix build .#game .#signaling
   ```

2. **Deploy to NixOS**:
   ```bash
   # Copy the built packages to your NixOS server
   scp result-* user@your-server:/tmp/
   
   # On the server, add to configuration.nix:
   services.hex-chess.enable = true;
   
   # Rebuild and switch
   sudo nixos-rebuild switch
   ```

## Hex Chess Variants

### Regular Hexagons
- **Gliński's Chess**: 91 cells, most popular variant
- **McCooey's Chess**: 81 cells, slightly smaller

### Irregular Boards
- **Shafran's Chess**: Custom irregular layout
- **Brusky's Chess**: Custom irregular layout  
- **De Vasa's Chess**: Custom irregular layout

### Small Boards
- **Mini Hexchess**: 37 cells, quick games

### Fairy Piece Variants
- **Gliński-Capablanca**: Gliński's board with Chancellor & Archbishop
- **McCooey-Capablanca**: McCooey's board with fairy pieces

## Technical Details

### Core Engine (`crates/core/`)
- **Hexagonal coordinates**: Axial coordinate system (q, r)
- **Piece movement**: Adapted for hex geometry with 6 directions + diagonals
- **Game rules**: Check/checkmate detection, move validation
- **Variants**: Data-driven configuration system

### Bevy Game (`crates/game/`)
- **Rendering**: 2D hex tiles with sprite system
- **Input**: Click-to-move with move highlighting
- **UI**: Variant selection, move history, game status
- **WASM**: Compiled to WebAssembly for browser deployment

### Signaling Server (`crates/signaling/`)
- **WebSocket**: Real-time communication for peer discovery
- **Room management**: Match players by variant
- **WebRTC**: SDP offer/answer exchange, ICE candidates
- **Lightweight**: ~50MB RAM, no game state storage

### Multiplayer Flow
1. Players connect to signaling server via WebSocket
2. Server matches players requesting same variant
3. WebRTC connection established (SDP exchange)
4. Game moves sent directly between peers (p2p)
5. Server only handles initial handshake

## Development Commands

```bash
# Build everything
nix build

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Build WASM
cd crates/game && trunk build

# Run signaling server
cd crates/signaling && cargo run

# Enter dev shell
nix develop
```

## Browser Support

- **Modern browsers**: Chrome 80+, Firefox 75+, Safari 13+
- **WebRTC**: Required for multiplayer
- **WebAssembly**: Required for game engine
- **WebGL**: Required for Bevy rendering

## Performance

- **WASM bundle**: ~2MB gzipped
- **Memory usage**: ~50MB in browser
- **Network**: Only signaling traffic, game moves are p2p
- **Rendering**: 60 FPS on modern hardware

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- [Green Chess](https://greenchess.net) for variant documentation
- [Bevy](https://bevyengine.org) for the game engine
- [WebRTC](https://webrtc.org) for p2p networking
- [Nix](https://nixos.org) for reproducible builds
