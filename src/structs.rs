pub struct GameInfo {
  pub id: i128,
  pub name: String,
  pub white: String,
  pub black: String,
  pub white_elo: i16,
  pub black_elo: i16,
  pub moves: String,
  pub date: String,
  pub time: String,
}

pub struct Evaluation {
  pub score: f64,
  // pub best_move: String,
  pub pv: Vec<String>,
  pub mate_in: i32
}

pub struct Puzzle {
  pub puzzle_idx: i128,
  pub game_idx: i128,
  pub start_pos: String,
  pub moves: Vec<String>,
  pub end_move: String,
  pub mate_in: i32,

  pub task: String, 
}