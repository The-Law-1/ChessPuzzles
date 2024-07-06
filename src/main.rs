mod evaluation;
mod structs;
mod parser;
mod serialise;

use std::{fs::OpenOptions, io::Write};

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

  let games_data : Vec<structs::GameInfo> = parser::parse_csv_games("./chess-games/data/2016_CvC.csv");

  let games_file = OpenOptions::new()
    .write(true)
    .open("./chess-games/data/2016_CvC.csv")?;

  // get the number of lines in the parsed_game_files
  let parsed_games = std::fs::read_to_string("parsed-games.txt")?;
  let parsed_games_count = parsed_games.lines().count();

  // run the engine
  let mut engine = evaluation::start_stockfish();

  let start_at = 1;
  let max_games = 10;

  let mut game_idx = 0;

  for game in games_data.iter() {
    if game_idx < start_at {
      println!("Game already parsed, skipping\n");
      game_idx += 1;
      continue;
    }

    if game.id >= max_games {
      break;
    }
    print!("Game: {}\n", game.name);

    print!("Testing game data: {}\n", game.name);

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

  // close the engine
  engine.kill().expect("Failed to kill Stockfish");
  println!("Closed stockfish\n");

  Ok(())
}