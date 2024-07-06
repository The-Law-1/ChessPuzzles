mod utils;

use crate::structs::{Evaluation, Puzzle};
use std::{env,  process::{Child, Command, Stdio}, str::FromStr};
use std::io::{Write, BufRead, BufReader};
use chess::{Board, ChessMove, Square};

pub fn start_stockfish() -> Child {
  // This function should start the Stockfish engine
  let engine_path = env::var("STOCKFISH_PATH").unwrap();

  let mut child = Command::new(engine_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to start Stockfish");

  let stdin = child.stdin.as_mut().expect("Failed to open stdin");
  let _stdout = child.stdout.as_mut().expect("Failed to open stdout");

  // Send UCI commands to Stockfish
  writeln!(stdin, "uci").expect("Failed to write to stdin");
  writeln!(stdin, "isready").expect("Failed to write to stdin");

  print!("Stockfish ready\n");
  return child;
}

// struct info


fn evaluate_position(fen: &str, engine: &mut Child) -> Vec<Evaluation> {
  let stdin = engine.stdin.as_mut().expect("Failed to open stdin");
  let stdout = engine.stdout.as_mut().expect("Failed to open stdout");
  let depth = 20;

  writeln!(stdin, "isready").expect("Failed to write to stdin");
  writeln!(stdin, "setoption name MultiPV value 2").expect("Failed to write to stdin");
  writeln!(stdin, "position fen {}", fen).expect("Failed to write to stdin");
  writeln!(stdin, "go depth {}", depth).expect("Failed to write to stdin");

  let mut evaluations : Vec<Evaluation> = Vec::new();
  let mut eval_idx: usize = 0;
  
  let reader = BufReader::new(stdout);
  let mut evaluation = 0.0;
  for line in reader.lines() {
    let line = line.expect("Failed to read line");
    if line.starts_with("bestmove") {
      break;
    }
    if line.contains(&("info depth ".to_string() + &depth.to_string())) {
      if evaluations.len() < eval_idx + 1 {
        evaluations.push(Evaluation {
          score: 0.0,
          pv: Vec::new(),
          mate_in: -1
        });
      }

      if line.contains(" pv ") {
        let pv = line.split(" pv ").nth(1).unwrap();
        evaluations[eval_idx].pv = pv.split_whitespace().map(|s| s.to_string()).collect();
      }

      if line.contains("score cp") {
        let score: i32 = line.split("score cp")
                              .nth(1)
                              .unwrap()
                              .split_whitespace()
                              .next()
                              .unwrap()
                              .parse()
                              .unwrap();
        evaluation = score as f64 / 100.0;
        evaluations[eval_idx].score = evaluation;
      }
      if line.contains("score mate") {
        let mate_in: i32 = line.split("score mate")
                              .nth(1)
                              .unwrap()
                              .split_whitespace()
                              .next()
                              .unwrap()
                              .parse()
                              .unwrap();
        
        // no need to give an eval, our algo will pick up the mate_in value
        evaluations[eval_idx].score = 0.0;
        evaluations[eval_idx].mate_in = mate_in;
      }

      eval_idx += 1;
    }
  }
  return evaluations;
}

fn explore_variation(start_pos: &String, moves: &Vec<String>, color: chess::Color, position_score: f64, max_depth : i16) -> (ChessMove, Vec<ChessMove>) {
  let mut best_moves = Vec::new();
  let mut last_winning_move = ChessMove::default();
  let mut board = Board::from_str(&start_pos).unwrap();
  let mut best_material_gain = 0;

  let opposing_color = if color == chess::Color::White { chess::Color::Black } else { chess::Color::White };

  let win_material_threshold = 2.5; // * we consider this a material-winning move

  let mut total_material_won = 0.0;

  for mv in moves {
    if (best_moves.len() as i16) >= max_depth {

      // if at the end of the sequence, we haven't won enough material, skip the puzzle
      if total_material_won < win_material_threshold {
        last_winning_move = ChessMove::default();
        return (last_winning_move, Vec::new());
      } else {
        return (last_winning_move, best_moves);
      }
    }

    // let static_eval_before = utils::material_points(&board, opposing_color);
    let chess_move = utils::convert_to_san(mv);
    board = board.make_move_new(chess_move);
    let static_eval_after = utils::material_points(&board, color);

    total_material_won += static_eval_after as f64;

    if color == chess::Color::White && total_material_won >= win_material_threshold {
      last_winning_move = chess_move;
    }
    if color == chess::Color::Black && total_material_won <= -win_material_threshold {
      last_winning_move = chess_move;
    }
    best_moves.push(chess_move);
  }

  (last_winning_move, best_moves)
}

pub fn find_tactical_positions(moves: &[String], engine: &mut Child) -> Vec<Puzzle> {
  let mut board = Board::default();
  
  let mut puzzles: Vec<Puzzle> = Vec::new();
  
  // Initialize with the evaluation of the initial position if available
  let mut prev_eval: Evaluation = Evaluation {
    score: 0.0,
    pv: Vec::new(),
    mate_in: -1
  };

  println!("Analyzing moves: {}", moves.len());

  for mv in moves {

    println!("Analyzing move: {}", mv);
    // TODO move forward to skip opening moves

    let mut mv = mv.replace("=", ""); // Remove the '=' sign from pawn promotions

    // ! handle castling with check O-O-O+ and O-O+
    mv = mv.replace("O-O-O+", "O-O-O");
    mv = mv.replace("O-O+", "O-O");

    // if it's the game's result, break
    if mv == "1-0" || mv == "0-1" || mv == "1/2-1/2" {
      break;
    }

    // if the move is a mate, it will crash stockfish
    if mv.contains("#") {
      break;
    }

    // find the en passant square
    let en_passant : Option<Square> = board.en_passant();

    // ! en passant moves will crash the library
    // https://github.com/jordanbray/chess/issues/54
    if en_passant.is_some() {
      let backward_move : Option<Square> = en_passant.unwrap().forward(board.side_to_move());

      // ! should work with both colours! https://docs.rs/chess/latest/chess/struct.Square.html#method.backward
      if (backward_move.is_some()) {

        let backward_move_str = backward_move.unwrap().to_string();
        println!("Backward to en passant square: {}", backward_move_str);
        
        // * if the move takes the backward square, append " e.p."
        // * split mvoe at x
        let mv_parts : Vec<&str> = mv.split("x").collect();
        if (mv_parts.len() > 1) {
          let last_part = mv_parts.last().unwrap();
          if last_part.contains(&backward_move_str) {
            mv.push_str(" e.p.");
          }
        }
      }
    }

    let chess_move = ChessMove::from_san(&board, &mv).unwrap();
    board = board.make_move_new(chess_move);
    let fen_after = board.to_string();

    let evals_after = evaluate_position(&fen_after, engine);

    // * can happen when there is no best move https://github.com/official-stockfish/Stockfish/discussions/5075
    if evals_after.len() == 0 {
      println!("No evaluation found for move: {} at fen {}", mv, fen_after);
      continue;
    }

    let tactical_move_threshold = 2.5; // about the value of a piece in centipawns

    let mut is_tactical_move = false;

    // * if this position swung the score by more than 2.5 centipawns, it's a tactical move
    if (prev_eval.score - evals_after[0].score).abs() > tactical_move_threshold {
      println!("Tactical move detected: {}", mv);
      is_tactical_move = true;
    }

    // * or if a forced mate is detected now and the previous move was not a mate
    if evals_after[0].mate_in > 0 && prev_eval.mate_in == -1 {
      println!("Mate in detected: {}", mv);
      is_tactical_move = true;
    }

    // * if the move is not tactical, skip it
    // ! weird that pv would be empty, but we've had errors
    if is_tactical_move == false || evals_after[0].pv.is_empty() {
      prev_eval = Evaluation{
        score: evals_after[0].score,
        pv: evals_after[0].pv.clone(), // Clone the vector to avoid moving it
        mate_in: evals_after[0].mate_in
      }; // Store the current evaluation for the next iteration
      continue;
    }

    let colour_to_play = board.side_to_move();

    // check the final evaluation of the position, is the only one that matters
    let only_winning_move = utils::is_only_winning_move(&evals_after, fen_after.clone());

    // println!("Was best move the only winning move: {}", only_winning_move);

    if only_winning_move{
      let eval_after = &evals_after[0];
      println!("Only winning move detected: {}", eval_after.pv[0]);

      // TODO check if the move was missed by the player, to eliminate trivial puzzles
      
      let max_puzzle_length = 5;

      // * if the evaluation is positive, it means we can win material
      if eval_after.score > 0.0 {
        
        // look for a sequence of moves that lead to (the biggest) material gain about equal to the evaluation
        // * while staying under the max depth
        let puzzle_variation = explore_variation(&fen_after, &eval_after.pv, colour_to_play, eval_after.score, max_puzzle_length);

        // * likely our engine found a position-improving sequence but not a clear material-winning move
        if puzzle_variation.0 == ChessMove::default() && puzzle_variation.1.len() == 0 {
          println!("No clear material-winning move found, skipping puzzle");
          continue;
        }

        let puzzle_start_pos = board.to_string();
        let puzzle_moves : Vec<String> = puzzle_variation.1.iter().map(|m| m.to_string()).collect();

        // todo cut off puzzle_moves at last winning move?

        let task = if colour_to_play == chess::Color::White { "White to win material" } else { "Black to win material" };

        println!("Found tactical puzzle: {}, {}", puzzle_start_pos.to_string(), puzzle_moves.join(" "));
        println!("Last material winning move: {}", puzzle_variation.0.to_string());

        let puzzle : Puzzle = Puzzle {
          puzzle_idx: puzzles.len() as i128,
          game_idx: 0,
          start_pos: puzzle_start_pos,
          moves: puzzle_moves,
          end_move: puzzle_variation.0.to_string(),
          task: task.to_string(),
          mate_in: -1
        };


        puzzles.push(puzzle);
      }

      let max_mate_depth = 7;
      // * if instead we have a positive mate, we can force a mate
      if eval_after.mate_in > 0 && eval_after.mate_in <= max_mate_depth as i32 {
        let move_coords = utils::chess_move_to_coordinate_notation(&chess_move);

        if eval_after.pv[0] != move_coords {
          // let startPos = fen_before;
          let start_pos = fen_after.clone();
          let moves = eval_after.pv.clone();

          let task = if colour_to_play == chess::Color::White { "White to force mate" } else { "Black to force mate" };

          println!("Player missed mate, found puzzle: {}, {}", start_pos.to_string(), moves.join(" "));

          let puzzle : Puzzle = Puzzle {
            puzzle_idx: puzzles.len() as i128,
            game_idx: 0,
            start_pos,
            moves: moves.clone(),
            end_move: moves.last().unwrap().to_string(),
            task: task.to_string(),
            mate_in: eval_after.mate_in
          };

          puzzles.push(puzzle);
        }
      }

      prev_eval = Evaluation{
        score: eval_after.score,
        pv: eval_after.pv.clone(), // Clone the vector to avoid moving it
        mate_in: eval_after.mate_in
      }; // Store the current evaluation for the next iteration
    }
  }

  return puzzles;
}