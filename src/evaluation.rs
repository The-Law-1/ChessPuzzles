mod utils;

use crate::structs::{Evaluation, Puzzle};
use std::{env,  process::{Child, Command, Stdio}, str::FromStr};
use std::io::{Write, BufRead, BufReader};
use chess::{Board, ChessMove};

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
  let mut eval_idx = 0;
  
  let reader = BufReader::new(stdout);
    let mut evaluation = 0.0;
    for line in reader.lines() {
      let line = line.expect("Failed to read line");
      if line.starts_with("bestmove") {
        break;
      }
      // last line:
      if line.contains(&("info depth ".to_string() + &depth.to_string())) {
        if evaluations.len() < eval_idx + 1 {
          evaluations.insert(0, Evaluation {
            score: 0.0,
            pv: Vec::new(),
            mate_in: -1
          });
        }

        if line.contains(" pv ") {
          let pv = line.split(" pv ").nth(1).unwrap();
          evaluations[0].pv = pv.split_whitespace().map(|s| s.to_string()).collect();
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
          evaluations[0].score = evaluation;
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
          // huge eval so that it is always seen as a better move
          evaluation = if mate_in > 0 { 10000 - mate_in } else { -10000 - mate_in } as f64;
          evaluations[0].score = evaluation;
          evaluations[0].mate_in = mate_in;
        }

        eval_idx += 1;
      }
    }
  return evaluations;
}

fn explore_variation(start_pos: &String, moves: &Vec<String>, color: chess::Color, position_score: f64, max_depth : i16) -> (ChessMove, Vec<ChessMove>) {
  let mut best_moves = Vec::new();
  let mut best_move = ChessMove::default();
  let mut board = Board::from_str(&start_pos).unwrap();
  let mut best_material_gain = 0;

  let opposing_color = if color == chess::Color::White { chess::Color::Black } else { chess::Color::White };

  for mv in moves {
    if (best_moves.len() as i16) >= max_depth { break; }

    let static_eval_before = utils::material_points(&board, opposing_color);
    let chess_move = utils::convert_to_san(mv);
    board = board.make_move_new(chess_move);
    let static_eval_after = utils::material_points(&board, opposing_color);

    if board.side_to_move() != color {
        let difference = (static_eval_after - static_eval_before).abs();
        if difference as f64 >= position_score && difference >= best_material_gain {
          best_move = chess_move;
          best_material_gain = difference;
        }
    }
    best_moves.push(chess_move);
  }

  (best_move, best_moves)
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

    let chess_move = ChessMove::from_san(&board, &mv).unwrap();
    let fen_before = board.to_string();
    board = board.make_move_new(chess_move);
    let fen_after = board.to_string();

    let evals_after = evaluate_position(&fen_after, engine);

    let tactical_move_threshold = 2.5; // about the value of a piece in centipawns


    let colour_to_play = board.side_to_move();

    // check the final evaluation of the position, is the only one that matters
    let only_winning_move = utils::is_only_winning_move(&evals_after, tactical_move_threshold);

    // println!("Was best move the only winning move: {}", only_winning_move);

    if only_winning_move {
      let eval_after = &evals_after[0];

      // TODO check if the move was missed by the player, to eliminate trivial puzzles


      // if the previous eval is already high, one side is winning, we don't need to check
      if prev_eval.score.abs() < tactical_move_threshold && (eval_after.score - prev_eval.score).abs() > tactical_move_threshold {

        
        // look for a sequence of moves that lead to (the biggest) material gain about equal to the evaluation
        let puzzle_variation = explore_variation(&fen_after, &eval_after.pv, colour_to_play, eval_after.score, 5);

        let puzzle_start_pos = board.to_string();
        let puzzle_moves : Vec<String> = puzzle_variation.1.iter().map(|m| m.to_string()).collect();

        let task = if colour_to_play == chess::Color::White { "White to win material" } else { "Black to win material" };

        println!("Found tactical puzzle: {}, {}", puzzle_start_pos.to_string(), puzzle_moves.join(" "));

        let puzzle : Puzzle = Puzzle {
          puzzle_idx: 0,
          game_idx: 0,
          start_pos: puzzle_start_pos,
          moves: puzzle_moves,
          task: task.to_string(),
          mate_in: -1
        };


        puzzles.push(puzzle);
      }

      if prev_eval.mate_in > 0 && prev_eval.mate_in <= 7 {
        let move_coords = utils::chess_move_to_coordinate_notation(&chess_move);

        if prev_eval.pv[0] != move_coords {
          // let startPos = fen_before;
          let start_pos = fen_before;
          let moves = prev_eval.pv.clone();

          let task = if colour_to_play == chess::Color::White { "White to force mate" } else { "Black to force mate" };

          println!("Found mate puzzle: {}, {}", start_pos.to_string(), moves.join(" "));

          let puzzle : Puzzle = Puzzle {
            puzzle_idx: 0,
            game_idx: 0,
            start_pos,
            moves: moves,
            task: task.to_string(),
            mate_in: prev_eval.mate_in
          };

          puzzles.push(puzzle);
          println!("Missed mate in in {}\n", prev_eval.mate_in);
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