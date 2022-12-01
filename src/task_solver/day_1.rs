use anyhow::{anyhow, Context, Result};
use log::info;
use std::{
    cmp,
    fs::File,
    io::{BufRead, BufReader},
};

pub fn solve(task: u8, input: String) -> Result<()> {
    match task {
        1 => solve_1(input),
        _ => Err(anyhow!("Haven't solved this task, yet.")),
    }
}

fn solve_1(input: String) -> Result<()> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut cur_cals = 0u32;
    let mut max_cals = 0u32;

    loop {
        let bytes_read = in_reader
            .read_line(&mut line)
            .context("Failed to read line in input file")?;
        if bytes_read == 0 {
            break; // EOF
        }

        if line == "\n" {
            max_cals = cmp::max(cur_cals, max_cals);
            cur_cals = 0;
        } else {
            let cals = line
                .trim()
                .parse::<u32>()
                .context("Failed to parse int from input")?;
            cur_cals += cals;
        }

        line.clear();
    }

    info!(
        "Highest number of calories carried by an elve: {}",
        max_cals
    );

    Ok(())
}
