use std::{fs::File, io::{Result, Write}};

use crate::structs::Puzzle;

fn write_to_file(file: &mut File, line: &str) -> Result<()> {
  file.write_fmt(format_args!("{}", line))
}

pub fn write_puzzles(file: &mut File, puzzles: Vec<Puzzle>) -> i16 {

  for puzzle in puzzles {
    let mut moves = String::new();
    for mv in puzzle.moves {
      moves.push_str(&mv);
      moves.push_str(" ");
    }
    let line = format!("{}, {}, {}, {}, {}, {}\n", puzzle.puzzle_idx, puzzle.game_idx, puzzle.start_pos, moves, puzzle.mate_in, puzzle.task);

    if let Err(e) = write_to_file(file, &line) {
      eprintln!("Failed to write to file: {}", e);
      return 84;
    }
  }
  return 0;
}