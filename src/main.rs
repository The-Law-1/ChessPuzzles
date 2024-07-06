mod evaluation;
mod structs;
mod parser;
mod serialise;

use std::fs::File;
use std::{fs::OpenOptions, io::Write};
use std::panic;

use dotenv;

use crate::structs::Puzzle;

// parse the moves into an array
fn parse_pgn(pgn: &str) -> Vec<String> {
  let mut moves: Vec<String> = Vec::new();
  for token in pgn.split_whitespace() {
      if token.contains('.') {
          continue; // Skip move numbers
      }
      moves.push(token.to_string());
  }
  moves
}

fn main() -> std::io::Result<()> {
  dotenv::dotenv().ok();

  panic::set_hook(Box::new(|panic_info| {
      let mut err_file = File::create("error.txt").expect("Unable to create file");
      write!(err_file, "Error: {}", panic_info).expect("Unable to write to file");
  }));

  let games_data : Vec<structs::GameInfo> = parser::parse_csv_games("./scripts/The_Lawx_games.csv");

  // run the engine
  let mut engine = evaluation::start_stockfish();

  let start_at = 0;
  let max_games = 100;

  let mut game_idx = 0;

  for game in games_data.iter() {
    if game_idx < start_at {
      println!("Game already parsed, skipping");
      game_idx += 1;
      continue;
    }

    if game.id >= max_games {
      break;
    }
    println!("Game: {}\n Number: {}", game.name, game_idx);

    println!("Testing game data: {}", game.name);

    let moves: Vec<String> = parse_pgn(game.moves.as_str());

    // open a file for writing
    let mut file = OpenOptions::new()
      .write(true)
      .create(true)
      .open("output.csv")?;

        
    if file.metadata()?.len() == 0 {
      println!("File is empty, writing headers");
      let _ = file.write_all(b"PuzzleIdx, GameIdx, Game, StartPos, Moves, MateIn, Task\n");
    }

    println!("Calling find_tactical_positions\n");
    let mut puzzles : Vec<Puzzle> = evaluation::find_tactical_positions(&moves, &mut engine);

    for puzzle in puzzles.iter_mut() {
      puzzle.game_idx = game_idx as i128;
    }

    let write_res = serialise::write_puzzles(&mut file, puzzles);
    if write_res != 0 {
      println!("Failed to write to file\n");
      // break loop
    }


    file.sync_all()?;
    println!("Wrote to puzzles file\n");

    // write to a file that contains the ids of the parsed games

    game_idx += 1;
  }


  // write what games you analyzed
  let mut parsed_games_file = OpenOptions::new()
    .write(true)
    .create(true)
    .open("parsed_games.csv")?;

  let line = format!("{}, {}\n", start_at, game_idx);
  if let Err(e) = parsed_games_file.write_fmt(format_args!("{}", line)){
    eprintln!("Failed to write to file: {}", e);
  }



  // close the engine
  engine.kill().expect("Failed to kill Stockfish");
  println!("Closed stockfish\n");

  Ok(())
}