#! /usr/bin/python3

import requests
from sys import argv
from datetime import datetime
import re

username = argv[1]
profile_endpoint = f"https://api.chess.com/pub/player/{username}"

month = "10"
year = "2009"
games_endpoint = f"https://api.chess.com/pub/player/{username}/games/{year}/{month}/pgn"

def parse_pgn(pgn):
  
  if (pgn == ""):
    return "", "", "", "", "", ""

  try:
    # split the pgn into lines
    lines = pgn.split("\n")
    
    white = re.search(r'\[White "(.*?)"\]', pgn).group(1)
    black = re.search(r'\[Black "(.*?)"\]', pgn).group(1)
    
    # [UTCDate]
    date = re.search(r'\[UTCDate "(.*?)"\]', pgn).group(1)
    
    # WhiteElo
    rating_white = re.search(r'\[WhiteElo "(.*?)"\]', pgn).group(1)
    # BlackElo
    rating_black = re.search(r'\[BlackElo "(.*?)"\]', pgn).group(1)
    
    moves = lines[-1]
    
    # parse moves from 1. c1 1... c5
    # to 1. c1 c5
    moves = re.sub(r'\d+\.\.\.', '', moves)
    # some whitespace remains, but it's not a big deal
    
    return white, black, date, rating_white, rating_black, moves

  except Exception as e:
    print(f"Error parsing pgn {e}")
    print(pgn)
    exit(1)

# https://www.chess.com/news/view/published-data-api#pubapi-endpoint-player
# fetch player endpoint
headers = {
    "User-Agent": "gabrielkgriffin@gmail.com",
    "email": "gabrielkgriffin@gmail.com",
    "Content-Type": "application/json",
}

response = requests.get(profile_endpoint, headers=headers)
if response.status_code == 200:
    profile = response.json()

    print(f"Joined: {profile['joined']}")
    joined_ts = profile['joined']
    
    # Convert the timestamp to a datetime object
    dt_object = datetime.fromtimestamp(joined_ts)
    month = dt_object.month
    year = dt_object.year
    print(f"Joined: {month}/{year}")
    
    #generate current month and year
    now = datetime.now()
    current_month = now.month
    current_year = now.year
    
    my_games_file = open(f"{username}_games.csv", "w")
    my_games_file.write(
      f"name,White,Black,White Elo,Black Elo,_,_,_,_,_,Date,Time,_,_,_,_,_,_,_,Moves\n"
    )

    while year < current_year or month < current_month:
        # https://www.chess.com/news/view/published-data-api#pubapi-endpoint-games-pgn
      
        games_endpoint = f"https://api.chess.com/pub/player/{username}/games/{year}/{month}/pgn"
        response = requests.get(games_endpoint, headers=headers)
        print(f"Month: {month}, Year: {year}")
        if response.status_code == 200:
          
          # print(response.text)
          pgns = response.text.split("\n\n\n")
         
          print(f"Played: {len(pgns)}")
          for pgn in pgns:
            white, black, date, rating_white, rating_black, moves = parse_pgn(pgn)
            if (white == ""):
              continue

            # watch for encoding in pgn
            # {[%clk 0:09:01.9]}
            moves = re.sub(r'\{.*?\}', '', moves)            
            my_games_file.write(
              f"{white} vs {black},{white},{black},{rating_white},{rating_black},_,_,_,_,_,{date},_,_,_,_,_,_,_,_,{moves}\n"
            )
        month += 1
        if month > 12:
            month = 1
            year += 1


    
    