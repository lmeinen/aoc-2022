use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use log::info;
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
};

pub fn solve(task: u8, input: String) -> Result<()> {
    // instantiate parser
    let parser = DataStreamParser::init(input).context("Failed to instantiate parser")?;

    let mut in_buff = VecDeque::new();

    for (c, i) in parser {
        if in_buff.len()
            == (match task {
                1 => 4,
                2 => 14,
                _ => panic!("task doesn't exist!"),
            })
        {
            let _ = in_buff.pop_front().unwrap();
            in_buff.push_back(c);
            if in_buff.iter().unique().count() == in_buff.len() {
                info!(
                    "first marker after character {} for marker {:?}",
                    i + 1,
                    in_buff
                );
                break;
            }
        } else {
            in_buff.push_back(c);
        }
    }

    Ok(())
}

struct DataStreamParser {
    char_list: Vec<char>,
    char_index: u32,
}

impl DataStreamParser {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let mut in_reader = BufReader::new(in_file);
        let mut line = String::new();
        let bytes_read = in_reader
            .read_line(&mut line)
            .expect("failed to read input file");
        if bytes_read == 0 {
            return Err(anyhow!("input file was empty!"));
        }

        Ok(DataStreamParser {
            char_list: line.chars().collect(),
            char_index: 0,
        })
    }
}

impl Iterator for DataStreamParser {
    type Item = (char, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let next_char = self.char_list.get(self.char_index as usize);
        if let Some(c) = next_char {
            let i = self.char_index;
            self.char_index += 1;
            Some((c.to_owned(), i))
        } else {
            None
        }
    }
}
