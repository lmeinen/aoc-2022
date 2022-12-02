use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{anyhow, Context, Result};
use log::info;

pub fn solve(task: u8, input: String) -> Result<()> {
    // initiate parser
    let parser = GuideParser::init(input).context("failed to instantiate parser")?;

    let mut total_score = 0u32;

    for round in parser {
        total_score += match task {
            1 => get_score_1(round.0.as_str(), round.1.as_str())
                .context("failed to compute score for round")?,
            2 => get_score_2(round.0.as_str(), round.1.as_str())
                .context("failed to compute score for round")?,
            _ => return Err(anyhow!("this task doesn't exist!")),
        };
    }

    info!("Strategy guide results in total score of {}", total_score);

    Ok(())
}

fn get_score_1(opp_choice: &str, player_choice: &str) -> Result<u32> {
    let init_i: u32 = match opp_choice {
        "A" => 1,
        "B" => 0,
        "C" => 2,
        _ => return Err(anyhow!("Illegal opp choice {}", opp_choice)),
    };
    let index = match player_choice {
        "X" => 0,
        "Y" => 1,
        "Z" => 2,
        _ => return Err(anyhow!("Illegal player choice {}", player_choice)),
    };
    Ok(((init_i + index) % 3) * 3 + index + 1)
}

fn get_score_2(opp_choice: &str, player_choice: &str) -> Result<u32> {
    let mut choice_i = match opp_choice {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        _ => return Err(anyhow!("Illegal opp choice {}", opp_choice)),
    };
    let result = match player_choice {
        "X" => {
            choice_i = (choice_i + 2) % 3;
            0
        }
        "Y" => 1,
        "Z" => {
            choice_i = (choice_i + 1) % 3;
            2
        }
        _ => return Err(anyhow!("Illegal player choice {}", player_choice)),
    };
    Ok(result * 3 + choice_i + 1)
}

struct GuideParser {
    in_reader: BufReader<File>,
    line: String,
}

impl GuideParser {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let in_reader = BufReader::new(in_file);
        let line = String::new();

        Ok(GuideParser { in_reader, line })
    }
}

impl Iterator for GuideParser {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let bytes_read = self
            .in_reader
            .read_line(&mut self.line)
            .expect("Failed to read line in input file");
        if bytes_read == 0 {
            return None; // EOF
        } else if self.line == "\n" {
            return None; // No more predictions to parse
        }

        let mut round = self.line.trim().split_whitespace();
        let opp_choice = round
            .next()
            .expect("line didn't contain prediction for opponent's choice")
            .to_owned();
        let my_choice = round
            .next()
            .expect("line didn't contain prediction for players's choice")
            .to_owned();
        if let Some(rem_prediction) = round.next() {
            panic!("line contained another element: {}", rem_prediction);
        }

        self.line.clear();
        Some((opp_choice, my_choice))
    }
}
