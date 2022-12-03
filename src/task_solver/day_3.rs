use core::panic;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result};
use log::{debug, info};

pub fn solve(task: u8, input: String) -> Result<()> {
    let rucksack_parser =
        RucksackParser::init(input, task).context("failed to instantiate parser")?;

    let mut item_sum = 0;
    for i in rucksack_parser {
        let prio = get_prio(i);
        item_sum += prio;
        debug!("Overlapping item: {} -> {}", i, prio);
    }

    info!(
        "Sum of the priorities of all erroneously-sorted items: {}",
        item_sum
    );
    Ok(())
}

fn get_prio(i: char) -> u16 {
    if i.is_lowercase() {
        i as u16 - 'a' as u16 + 1
    } else {
        i as u16 - 'A' as u16 + 27
    }
}

fn slices_common_item(rucksack_slices: &[&str]) -> Option<char> {
    if let Some(first) = rucksack_slices.first() {
        let others = &rucksack_slices[1..];
        first
            .chars()
            .find(|&c| others.iter().all(|s| s.contains(c)))
    } else {
        None
    }
}

struct RucksackParser {
    in_reader: BufReader<File>,
    line: String,
    task: u8,
}

impl RucksackParser {
    fn init(input: String, task: u8) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let in_reader = BufReader::new(in_file);
        let line = String::new();

        Ok(RucksackParser {
            in_reader,
            line,
            task,
        })
    }
}

impl Iterator for RucksackParser {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        while !match self.task {
            1 => self.line.split_terminator('\n').count() == 1,
            2 => self.line.split_terminator('\n').count() == 3,
            _ => panic!("task doesn't exist!"),
        } {
            let bytes_read = self
                .in_reader
                .read_line(&mut self.line)
                .expect("Failed to read line in input file");
            if bytes_read == 0 {
                return None; // EOF
            } else if self.line == "\n" {
                return None; // No more rucksacks to parse
            }
        }

        let rucksack_slices = match self.task {
            1 => {
                let len = self.line.len();
                vec![&self.line[..len / 2], &self.line[len / 2..]]
            }
            2 => {
                let tmp_vec: Vec<&str> = self.line.split_terminator('\n').collect();
                debug!("rucksack contains items: {:?}", tmp_vec);
                tmp_vec
            }
            _ => panic!("task doesn't exist!"),
        };

        let shared_item = slices_common_item(rucksack_slices.as_slice())
            .expect("Rucksack compartments didn't contain overlapping item!");

        self.line.clear();
        Some(shared_item)
    }
}
