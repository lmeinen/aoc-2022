use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::{
    cmp,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::task_solver::util;

pub fn solve(task: u8, input: String) -> Result<()> {
    match task {
        1 => solve_1(input),
        2 => solve_2(input),
        _ => Err(anyhow!("This task doesn't exist - choose one of 1 or 2.")),
    }
}

fn solve_1(input: String) -> Result<()> {
    // instantiate parser
    let parser = ElfParser::init(input).context("Failed to instantiate parser")?;

    let mut max_cals = 0u32;

    for cur_cals in parser {
        max_cals = cmp::max(cur_cals, max_cals);
    }

    info!("Highest number of calories carried by an elf: {}", max_cals);

    Ok(())
}

fn solve_2(input: String) -> Result<()> {
    // instantiate parser
    let parser = ElfParser::init(input).context("Failed to instantiate parser")?;

    let mut sorted_list = util::SortedList::<u32>::new(3);

    for elf_cals in parser {
        debug!("Checking for value {}...", elf_cals);
        sorted_list.insert(elf_cals);
    }

    let total = sorted_list.fold::<u32>(0u32, &|cals, sum| cals + sum);

    info!(
        "Total number of calories carried by the three elves carrying the most calories: {}",
        total
    );

    Ok(())
}

struct ElfParser {
    in_reader: BufReader<File>,
    line: String,
}

impl ElfParser {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let in_reader = BufReader::new(in_file);
        let line = String::new();

        Ok(ElfParser { in_reader, line })
    }
}

impl Iterator for ElfParser {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut cur_cals = 0u32;
        loop {
            let bytes_read = self
                .in_reader
                .read_line(&mut self.line)
                .expect("Failed to read line in input file");
            if bytes_read == 0 {
                return None; // EOF
            }

            if self.line == "\n" {
                break;
            } else {
                let cals = self
                    .line
                    .trim()
                    .parse::<u32>()
                    .expect("Failed to parse int from input");
                self.line.clear();
                cur_cals += cals;
            }
        }
        Some(cur_cals)
    }
}
