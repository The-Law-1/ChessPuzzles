use std::str::FromStr;

use chess::{Board, ChessMove, Piece, Square, ALL_SQUARES};
use super::Evaluation;

// * https://github.com/jordanbray/chess/issues/25
pub fn material_points(board: &Board, color: chess::Color) -> i16 {
  let b = board.color_combined(color).0;
  let mut sum = 0i16;

  for i in 0..64u64 {
      if b & (1 << i) != 0 {
          let a:Square = ALL_SQUARES[i as usize];
          if let Some(piece) = board.piece_on(a) {
              sum += match piece {
                  Piece::Pawn => {1},
                  Piece::Knight => {3},
                  Piece::Bishop => {3},
                  Piece::Rook => {5},
                  Piece::Queen => {9},
                  Piece::King => {100},
              }
          }
      }
  }
  // return sum;
  match color {
    chess::Color::White => return sum,
    chess::Color::Black => return -sum,
  }
}

pub fn convert_to_san(move_str: &str) -> ChessMove {
  let from_square = Square::from_str(&move_str[0..2]).unwrap();
  let to_square = Square::from_str(&move_str[2..4]).unwrap();
  let promotion = if move_str.len() > 4 {
      match move_str.chars().nth(4).unwrap() {
          'q' => Some(Piece::Queen),
          'r' => Some(Piece::Rook),
          'b' => Some(Piece::Bishop),
          'n' => Some(Piece::Knight),
          _ => None,
      }
  } else {
    None
  };
  let chess_move = ChessMove::new(from_square, to_square, promotion);

  return chess_move;
}

// * we can assume since we check for tactical moves, that the best move is a winning move
pub fn is_only_winning_move(evals : &Vec<Evaluation>, current_fen : String) -> bool {
  let mut others_losing = true; // ! not a great coding pattern
 
  if evals.len() == 0 {
    println!("No evaluations found");
    return false;
  }

  // loop through the rest of the evaluations
  for eval in evals.iter().skip(1) {
    if evals[0].mate_in != -1 && (eval.mate_in == -1 || eval.mate_in > evals[0].mate_in) {
      // if the best move is a better mate in X, continue
      continue;
    }

    let less_winning_threshold = 0.5;
    let score_diff = (evals[0].score - eval.score).abs();

    // if no PV leads to mate, compare scores
    if (evals[0].mate_in == -1 && eval.mate_in == -1) && score_diff < less_winning_threshold {
      println!("Current fen: {}", current_fen);
      println!("Failed because of: {} or {}", eval.score, eval.mate_in);
      println!("And best move: {} or {}", evals[0].score, evals[0].mate_in);
      others_losing = false;
    }
  }

  return others_losing;
}

pub fn chess_move_to_coordinate_notation(chess_move: &ChessMove) -> String {
  let from = chess_move.get_source();
  let to = chess_move.get_dest();
  format!("{}{}", from, to)
}