use anyhow::{bail, Context, Result};

use log::info;

use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
};

pub fn solve(task: u8, input: String) -> Result<()> {
    let (mut val_list, mut index_list) = parse_input(input).context("failed to parse input")?;
    let len = val_list.len();

    let (decryption_key, num_mixes) = match task {
        1 => (1, 1),
        2 => (811589153, 10),
        _ => bail!("task doesn't exist!"),
    };

    val_list.iter_mut().for_each(|v| *v = *v * decryption_key);

    for _ in 0..num_mixes {
        for index_i in 0..index_list.len() {
            let curr_i = index_list[index_i];
            let move_value = val_list.remove(curr_i);
            let dest_i = find_destination(move_value, len, curr_i);
            val_list.insert(dest_i, move_value);
            for i in index_list.iter_mut() {
                if dest_i < curr_i && dest_i <= *i && *i < curr_i {
                    *i += 1;
                } else if dest_i > curr_i && curr_i < *i && *i <= dest_i {
                    *i -= 1;
                }
            }
            index_list[index_i] = dest_i;
        }
    }

    let zero_index = val_list
        .iter()
        .enumerate()
        .find(|(_, &v)| v == 0)
        .map(|(i, _)| i)
        .unwrap();

    let first = val_list[(zero_index + 1000) % len];
    let second = val_list[(zero_index + 2000) % len];
    let third = val_list[(zero_index + 3000) % len];
    let sum = first + second + third;

    info!(
        "sum of grove coordinates: {} + {} + {} = {}",
        first, second, third, sum
    );

    Ok(())
}

fn find_destination(move_val: i64, len: usize, curr_i: usize) -> usize {
    let modular_move = move_val % (len as i64 - 1);
    let mut dest_i = curr_i as i64 + modular_move;
    if dest_i < 0 {
        dest_i = len as i64 + dest_i - 1;
    } else if dest_i >= len as i64 - 1 {
        dest_i = dest_i - len as i64 + 1;
    }
    dest_i as usize
}

fn parse_input(input: String) -> Result<(Vec<i64>, VecDeque<usize>)> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut val_list = Vec::new();
    let mut index_list = VecDeque::new();

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        index_list.push_back(val_list.len());
        val_list.push(line.trim().parse().unwrap());

        line.clear();
    }

    Ok((val_list, index_list))
}
