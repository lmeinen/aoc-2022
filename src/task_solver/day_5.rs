use core::panic;
use std::{
    collections::{hash_map::RandomState, HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use regex::Regex;

pub fn solve(task: u8, input: String) -> Result<()> {
    // initiate parser
    let mut crate_stacks = HashMap::new();
    let parser = CargoCraneParser::init(input, &mut crate_stacks, task)
        .context("failed to instantiate parser")?;

    debug!("Initialized cargo stack: {:?}", parser.crate_stacks);

    if parser.last().is_some() {
        debug!("final stack: {:?}", crate_stacks);
        let mut final_top = String::from("");
        for i in 1..10 {
            final_top.push(
                crate_stacks
                    .get(&i)
                    .expect(format!("crate stack {} is missing!", i).as_str())
                    .back()
                    .expect(format!("crate stack {} is empty!", i).as_str())
                    .to_owned(),
            );
        }
        info!("Final top crates: {}", final_top);
        Ok(())
    } else {
        Err(anyhow!("Input didn't contain any move operations!"))
    }
}

struct CargoCraneParser<'a> {
    in_reader: BufReader<File>,
    line: String,
    crate_stacks: &'a mut HashMap<u32, VecDeque<char>, RandomState>,
    task: u8,
}

impl<'a> CargoCraneParser<'a> {
    fn init(
        input: String,
        crate_stacks: &'a mut HashMap<u32, VecDeque<char>, RandomState>,
        task: u8,
    ) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let mut in_reader = BufReader::new(in_file);
        let mut line = String::new();

        // parse the initial crate stacks
        crate_stacks.insert(1, VecDeque::from(['H', 'R', 'B', 'D', 'Z', 'F', 'L', 'S']));
        crate_stacks.insert(2, VecDeque::from(['T', 'B', 'M', 'Z', 'R']));
        crate_stacks.insert(3, VecDeque::from(['Z', 'L', 'C', 'H', 'N', 'S']));
        crate_stacks.insert(4, VecDeque::from(['S', 'C', 'F', 'J']));
        crate_stacks.insert(5, VecDeque::from(['P', 'G', 'H', 'W', 'R', 'Z', 'B']));
        crate_stacks.insert(6, VecDeque::from(['V', 'J', 'Z', 'G', 'D', 'N', 'M', 'T']));
        crate_stacks.insert(7, VecDeque::from(['G', 'L', 'N', 'W', 'F', 'S', 'P', 'Q']));
        crate_stacks.insert(8, VecDeque::from(['M', 'Z', 'R']));
        crate_stacks.insert(9, VecDeque::from(['M', 'C', 'L', 'G', 'V', 'R', 'T']));

        loop {
            let bytes_read = in_reader
                .read_line(&mut line)
                .expect("Failed to read line in input file");
            if bytes_read == 0 {
                panic!("nothing to read!");
            } else if line == "\n" {
                line.clear();
                break;
            }
            line.clear();
        }

        Ok(CargoCraneParser {
            in_reader,
            line,
            crate_stacks,
            task,
        })
    }
}

impl<'a> Iterator for CargoCraneParser<'a> {
    type Item = ();

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

        // Apply move operation
        let re = Regex::new(r"move (\d+) from (\d) to (\d)").unwrap();
        let caps = re.captures(self.line.as_str()).unwrap();

        let from_stack = caps.get(2).unwrap().as_str().parse::<u32>().unwrap();
        let to_stack = caps.get(3).unwrap().as_str().parse::<u32>().unwrap();
        let mut move_stack = VecDeque::new();
        for _ in 0..caps.get(1).unwrap().as_str().parse::<u32>().unwrap() {
            let cargo_box = self
                .crate_stacks
                .get_mut(&from_stack)
                .expect(format!("crate stack {} is missing!", from_stack).as_str())
                .pop_back()
                .expect(format!("crate stack {} is empty!", from_stack).as_str());
            match self.task {
                1 => move_stack.push_back(cargo_box),
                2 => move_stack.push_front(cargo_box),
                _ => panic!("task doesn't exist!"),
            }
        }
        self.crate_stacks
            .get_mut(&to_stack)
            .expect(format!("crate stack {} is missing!", to_stack).as_str())
            .append(&mut move_stack);

        self.line.clear();
        Some(())
    }
}
