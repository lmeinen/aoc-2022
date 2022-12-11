use anyhow::{anyhow, Context, Result};
use core::panic;
use log::{debug, info};
use regex::Regex;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

pub fn solve(_task: u8, input: String) -> Result<()> {
    let parser = MoveParser::init(input).context("failed to instantiate parser")?;

    let (mut head_pos, mut tail_pos) = ((0i32, 0i32), (0i32, 0i32));
    let rope_len = match _task {
        1 => 2,
        2 => 10,
        _ => return Err(anyhow!("task doesn't exist!")),
    };
    let mut rope_accum: Vec<(i32, i32)> = (1..rope_len).map(|_| (0i32, 0i32)).collect(); // [H-1, ..., T]
    let mut visited = HashSet::new();
    visited.insert(tail_pos);

    for (dir, num) in parser {
        debug!("");
        debug!("--------- {:?} {} ---------", dir, num);
        for _ in 0..num {
            let mut curr_motion = dir.get_motion();
            head_pos = update_pos(head_pos, curr_motion);
            for accum in rope_accum.iter_mut() {
                // update accum
                let accum_new = (accum.0 + curr_motion.0, accum.1 + curr_motion.1);
                // check if move required
                if accum_new.0.abs() > 1 || accum_new.1.abs() > 1 {
                    curr_motion = (accum_new.0.clamp(-1, 1), accum_new.1.clamp(-1, 1));
                    *accum = (accum_new.0 - curr_motion.0, accum_new.1 - curr_motion.1);
                } else {
                    curr_motion = (0, 0);
                    *accum = accum_new;
                }
            }
            tail_pos = update_pos(tail_pos, curr_motion);
            let is_new = visited.insert(tail_pos);
            if is_new {
                debug!("MOVED TAIL BY {:?}", curr_motion);
            }

            debug!(
                "H: {:?}; T: {:?}; visited: {}",
                head_pos,
                tail_pos,
                visited.len()
            );
        }
    }

    info!("number of visited fields: {}", visited.len());

    Ok(())
}

fn update_pos((x, y): (i32, i32), (m_x, m_y): (i32, i32)) -> (i32, i32) {
    (x + m_x, y + m_y)
}

#[derive(Debug)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Direction {
    fn get_motion(&self) -> (i32, i32) {
        match self {
            Direction::UP => (1, 0),
            Direction::DOWN => (-1, 0),
            Direction::LEFT => (0, -1),
            Direction::RIGHT => (0, 1),
        }
    }
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Direction::UP),
            "D" => Ok(Direction::DOWN),
            "L" => Ok(Direction::LEFT),
            "R" => Ok(Direction::RIGHT),
            _ => Err(()),
        }
    }
}

struct MoveParser {
    in_reader: BufReader<File>,
    line: String,
}

impl MoveParser {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let in_reader = BufReader::new(in_file);
        let line = String::new();

        Ok(MoveParser { in_reader, line })
    }
}

impl Iterator for MoveParser {
    type Item = (Direction, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let re_move = Regex::new(r"^(?P<dir>[UDRL]) (?P<num>\d+)").unwrap();

        let bytes_read = self
            .in_reader
            .read_line(&mut self.line)
            .expect("Failed to read line in input file");
        if bytes_read == 0 || self.line == "\n" {
            return None; // EOF
        }

        if re_move.is_match(&self.line) {
            let move_captures = re_move
                .captures(&self.line)
                .expect("move regex failed to capture line");
            let dir_str = move_captures
                .name("dir")
                .expect("move regex didn't contain expected named capture group")
                .as_str();
            let dir = Direction::from_str(dir_str)
                .expect("failed to parse direction from captured string");

            let num = move_captures
                .name("num")
                .expect("move regex didn't contain expected named capture group")
                .as_str()
                .parse::<u8>()
                .expect("failed to parse u8 number from captured string");

            self.line.clear();

            Some((dir, num))
        } else {
            panic!("line didn't match regex: {}", self.line);
        }
    }
}
