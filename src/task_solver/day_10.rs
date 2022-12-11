use anyhow::{anyhow, Context, Result};
use core::panic;
use log::{debug, info};
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub fn solve(task: u8, input: String) -> Result<()> {
    let parser = IParser::init(input).context("failed to instantiate parser")?;

    match task {
        1 => solve_1(parser),
        2 => solve_2(parser),
        _ => Err(anyhow!("task doesn't exist!")),
    }
}

fn solve_1(parser: IParser) -> Result<()> {
    let mut regx_sum = 0i32;
    for (cycle, regx) in parser {
        if cycle == 20 || (cycle as i32 - 20) % 40 == 0 {
            regx_sum += regx * cycle as i32;
            debug!(
                "cycle {}: adding {} * {} = {}",
                cycle,
                regx,
                cycle,
                regx * cycle as i32
            );
        }
    }

    info!("sum of signal strengths: {}", regx_sum);

    Ok(())
}

fn solve_2(parser: IParser) -> Result<()> {
    let mut curr_line = String::new();
    for (cycle, regx) in parser {
        if ((cycle as i32 - 1) % 40 - regx).abs() <= 1 {
            curr_line.push('#');
        } else {
            curr_line.push('.');
        }
        if cycle % 40 == 0 {
            info!("{}", curr_line);
            curr_line.clear();
        }
    }

    Ok(())
}

struct IParser {
    in_reader: BufReader<File>,
    reg_x: i32,
    cycle: u32,
    addx_val: Option<i32>,
    line: String,
}

impl IParser {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let in_reader = BufReader::new(in_file);
        let line = String::new();

        Ok(IParser {
            in_reader,
            reg_x: 1,
            cycle: 0,
            addx_val: None,
            line,
        })
    }
}

impl Iterator for IParser {
    type Item = (u32, i32); // (cycle, value of register X)

    fn next(&mut self) -> Option<Self::Item> {
        self.cycle += 1;

        let re_noop = Regex::new(r"^noop").unwrap();
        let re_addx = Regex::new(r"^addx (?P<num>-?\d+)").unwrap();

        if let Some(addx_val) = self.addx_val {
            let old_reg_x = self.reg_x;
            self.reg_x += addx_val;
            self.addx_val = None;
            Some((self.cycle, old_reg_x))
        } else {
            let bytes_read = self
                .in_reader
                .read_line(&mut self.line)
                .expect("Failed to read line in input file");
            if bytes_read == 0 || self.line == "\n" {
                return None; // EOF
            }

            if re_addx.is_match(&self.line) {
                let addx_captures = re_addx
                    .captures(&self.line)
                    .expect("addx regex failed to capture line");
                let addx_val_str = &addx_captures
                    .name("num")
                    .expect("addx regex didn't contain expected named capture group")
                    .as_str();
                let addx_val = addx_val_str
                    .parse::<i32>()
                    .expect("failed to parse i32 number from captured string");

                self.addx_val = Some(addx_val);

                self.line.clear();

                Some((self.cycle, self.reg_x))
            } else if re_noop.is_match(&self.line) {
                // 1 cycle passes
                self.line.clear();
                Some((self.cycle, self.reg_x))
            } else {
                panic!("line didn't match regex: {}", self.line);
            }
        }
    }
}
