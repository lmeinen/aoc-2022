use std::{
    fs::File,
    io::{BufRead, BufReader},
    ops::Range,
};

use anyhow::{anyhow, Context, Result};
use log::{debug, info};

pub fn solve(task: u8, input: String) -> Result<()> {
    // initiate parser
    let parser = RangePairParser::init(input).context("failed to instantiate parser")?;

    let mut total_score = 0u32;

    for range_pair in parser {
        total_score += match task {
            1 => get_score_1(range_pair),
            2 => get_score_2(range_pair),
            _ => return Err(anyhow!("this task doesn't exist!")),
        };
    }

    info!("Strategy guide results in total score of {}", total_score);

    Ok(())
}

fn get_score_1((range_1, range_2): (Range<u32>, Range<u32>)) -> u32 {
    if (range_1.start <= range_2.start && range_1.end >= range_2.end)
        || (range_1.start >= range_2.start && range_1.end <= range_2.end)
    {
        1
    } else {
        0
    }
}

fn get_score_2((range_1, range_2): (Range<u32>, Range<u32>)) -> u32 {
    if (range_1.start <= range_2.start && range_1.end >= range_2.start)
        || (range_1.start >= range_2.start && range_1.start <= range_2.end)
    {
        1
    } else {
        0
    }
}
struct RangePairParser {
    in_reader: BufReader<File>,
    line: String,
}

impl RangePairParser {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let in_reader = BufReader::new(in_file);
        let line = String::new();

        Ok(RangePairParser { in_reader, line })
    }
}

fn parse_range(range: &str) -> Range<u32> {
    debug!("parsing range {}", range);
    let range_split: Vec<&str> = range.split('-').collect();
    if range_split.len() == 2 {
        Range {
            start: range_split[0]
                .parse::<u32>()
                .expect("failed to parse start of range"),
            end: range_split[1]
                .parse::<u32>()
                .expect("failed to parse end of range"),
        }
    } else {
        panic!("Input range didn't contain exactly 2 elements");
    }
}

impl Iterator for RangePairParser {
    type Item = (Range<u32>, Range<u32>);

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

        debug!("parsing line {}", self.line);

        let range_pair: Vec<&str> = self.line.trim().split(',').collect();
        let range_pair_parsed;
        if range_pair.len() == 2 {
            range_pair_parsed = (parse_range(range_pair[0]), parse_range(range_pair[1]));
        } else {
            panic!("line didn't contain exactly 2 range elemens");
        }

        self.line.clear();
        Some(range_pair_parsed)
    }
}
