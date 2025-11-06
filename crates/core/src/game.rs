use crate::coords::HexCoord;
use crate::board::{Board, BoardError};
use crate::pieces::{Piece, Color};
use crate::variants::VariantConfig;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub board: Board,
    pub current_player: Color,
    pub move_history: VecDeque<Move>,
    pub game_state: GameState,
    pub variant: VariantConfig,
}

/// Current state of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Playing,
    Check(Color),      // Which color is in check
    Checkmate(Color),  // Which color is checkmated
    Stalemate,
    Draw,
}

/// A move in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub from: HexCoord,
    pub to: HexCoord,
    pub piece: Piece,
    pub captured_piece: Option<Piece>,
    pub move_number: u32,
}

impl Game {
    /// Create a new game with the given variant
    pub fn new(variant: VariantConfig) -> Self {
        let board = variant.create_board();
        
        Self {
            board,
            current_player: Color::White,
            move_history: VecDeque::new(),
            game_state: GameState::Playing,
            variant,
        }
    }

    /// Make a move
    pub fn make_move(&mut self, from: HexCoord, to: HexCoord) -> Result<(), GameError> {
        // Validate the move
        self.validate_move(from, to)?;
        
        // Get the piece being moved
        let piece = self.board.get_piece(from)
            .ok_or(GameError::NoPieceAtCoordinate)?
            .clone();
        
        // Check if there's a piece to capture
        let captured_piece = self.board.get_piece(to).cloned();
        
        // Make the move
        self.board.move_piece(from, to)?;
        
        // Record the move
        let move_number = (self.move_history.len() / 2) as u32 + 1;
        let game_move = Move {
            from,
            to,
            piece,
            captured_piece,
            move_number,
        };
        self.move_history.push_back(game_move);
        
        // Switch players
        self.current_player = match self.current_player {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        
        // Update game state
        self.update_game_state();
        
        Ok(())
    }

    /// Validate a move
    fn validate_move(&self, from: HexCoord, to: HexCoord) -> Result<(), GameError> {
        // Check if there's a piece at the source
        let piece = self.board.get_piece(from)
            .ok_or(GameError::NoPieceAtCoordinate)?;
        
        // Check if it's the current player's piece
        if piece.color != self.current_player {
            return Err(GameError::NotYourPiece);
        }
        
        // Check if the move is valid for the piece
        let valid_moves = self.board.get_valid_moves(from);
        if !valid_moves.contains(&to) {
            return Err(GameError::InvalidMove);
        }
        
        // Check if the move would put own king in check
        let test_board = self.board.with_move(from, to)?;
        if self.is_king_in_check(&test_board, self.current_player) {
            return Err(GameError::MoveWouldPutKingInCheck);
        }
        
        Ok(())
    }

    /// Check if a king is in check
    fn is_king_in_check(&self, board: &Board, color: Color) -> bool {
        let king_pos = match board.get_king(color) {
            Some(pos) => pos,
            None => return false, // No king found
        };
        
        // Check if any opponent piece can attack the king
        let opponent_color = match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        
        for (coord, piece) in board.get_pieces_by_color(opponent_color) {
            if piece.piece_type.get_moves(coord, board).contains(&king_pos) {
                return true;
            }
        }
        
        false
    }

    /// Check if a player is in checkmate
    fn is_checkmate(&self, color: Color) -> bool {
        if !self.is_king_in_check(&self.board, color) {
            return false;
        }
        
        // Check if any move can get out of check
        for (coord, _piece) in self.board.get_pieces_by_color(color) {
            let valid_moves = self.board.get_valid_moves(coord);
            for target in valid_moves {
                let test_board = self.board.with_move(coord, target).unwrap();
                if !self.is_king_in_check(&test_board, color) {
                    return false; // Found a move that gets out of check
                }
            }
        }
        
        true
    }

    /// Check if a player is in stalemate
    fn is_stalemate(&self, color: Color) -> bool {
        if self.is_king_in_check(&self.board, color) {
            return false; // Can't be stalemate if in check
        }
        
        // Check if any move is possible
        for (coord, _piece) in self.board.get_pieces_by_color(color) {
            let valid_moves = self.board.get_valid_moves(coord);
            for target in valid_moves {
                let test_board = self.board.with_move(coord, target).unwrap();
                if !self.is_king_in_check(&test_board, color) {
                    return false; // Found a valid move
                }
            }
        }
        
        true
    }

    /// Update the game state based on current position
    fn update_game_state(&mut self) {
        if self.is_checkmate(self.current_player) {
            let winner = match self.current_player {
                Color::White => Color::Black,
                Color::Black => Color::White,
            };
            self.game_state = GameState::Checkmate(winner);
        } else if self.is_stalemate(self.current_player) {
            self.game_state = GameState::Stalemate;
        } else if self.is_king_in_check(&self.board, self.current_player) {
            self.game_state = GameState::Check(self.current_player);
        } else {
            self.game_state = GameState::Playing;
        }
    }

    /// Get all valid moves for the current player
    pub fn get_valid_moves(&self) -> Vec<(HexCoord, Vec<HexCoord>)> {
        let mut moves = Vec::new();
        
        for (coord, _piece) in self.board.get_pieces_by_color(self.current_player) {
            let piece_moves = self.board.get_valid_moves(coord);
            if !piece_moves.is_empty() {
                moves.push((coord, piece_moves));
            }
        }
        
        moves
    }

    /// Undo the last move
    pub fn undo_move(&mut self) -> Result<(), GameError> {
        let last_move = self.move_history.pop_back()
            .ok_or(GameError::NoMovesToUndo)?;
        
        // Move the piece back
        self.board.move_piece(last_move.to, last_move.from)?;
        
        // Restore captured piece if any
        if let Some(captured) = last_move.captured_piece {
            self.board.place_piece(last_move.to, captured)?;
        }
        
        // Switch players back
        self.current_player = match self.current_player {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        
        // Update game state
        self.update_game_state();
        
        Ok(())
    }

    /// Get the game result as a string
    pub fn get_result(&self) -> Option<String> {
        match self.game_state {
            GameState::Checkmate(winner) => {
                let winner_name = match winner {
                    Color::White => "White",
                    Color::Black => "Black",
                };
                Some(format!("{} wins by checkmate", winner_name))
            }
            GameState::Stalemate => Some("Draw by stalemate".to_string()),
            GameState::Draw => Some("Draw".to_string()),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("No piece at the specified coordinate")]
    NoPieceAtCoordinate,
    #[error("Not your piece")]
    NotYourPiece,
    #[error("Invalid move")]
    InvalidMove,
    #[error("Move would put king in check")]
    MoveWouldPutKingInCheck,
    #[error("No moves to undo")]
    NoMovesToUndo,
    #[error("Board error: {0}")]
    BoardError(#[from] BoardError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variants::Variants;

    #[test]
    fn test_game_creation() {
        let variant = Variants::mini_hexchess();
        let game = Game::new(variant);
        assert_eq!(game.current_player, Color::White);
        assert_eq!(game.game_state, GameState::Playing);
    }

    #[test]
    fn test_move_validation() {
        let variant = Variants::mini_hexchess();
        let game = Game::new(variant);
        
        // Try to move a piece that doesn't exist
        let result = game.validate_move(HexCoord::new(0, 0), HexCoord::new(1, 0));
        assert!(result.is_err());
    }
}
