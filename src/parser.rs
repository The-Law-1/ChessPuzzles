use crate::structs::GameInfo;
use std::fs::File;

pub fn parse_csv_games(path: &str) -> Vec<GameInfo> {
  let mut games: Vec<GameInfo> = Vec::new();
  let file = File::open(path).expect("Failed to open file");
  let mut rdr: csv::Reader<File> = csv::Reader::from_reader(file);

  let mut index = 0;
  for result in rdr.records() {
    let record = result.expect("Failed to parse record");
    // println!("{:?}", record);
    let game = GameInfo {
      id: index,
      name: record[0].to_string(),
      white: record[1].to_string(),
      black: record[2].to_string(),
      white_elo: record[3].parse().unwrap(),
      black_elo: record[4].parse().unwrap(),
      date: record[10].to_string(),
      time: record[11].to_string(),
      moves: record[19].to_string()
    };
    index += 1;
    games.push(game);
  }
  games
}