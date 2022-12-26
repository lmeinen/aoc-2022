use anyhow::{bail, Context, Result};

use itertools::Itertools;
use log::{debug, info};

use std::{
    cmp,
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
};

use super::util::Point;

type Coordinate = Point<i32>;

pub fn solve(task: u8, input: String) -> Result<()> {
    let (mut elf_positions, mut left_top, mut right_bottom) =
        parse_input(input).context("failed to parse input")?;
    let directions: Vec<_> = vec![(-1, 0), (1, 0), (0, -1), (0, 1)]
        .into_iter()
        .map(|d| Coordinate::of_tuple(d))
        .collect();

    debug!("== Initial State ==");
    debug_region(&elf_positions, left_top, right_bottom);

    for r in 0.. {
        if r == 10 {
            let num_empty_tiles = (right_bottom.0.abs_diff(left_top.0) + 1)
                * (right_bottom.1.abs_diff(left_top.1) + 1)
                - elf_positions.len() as u32;

            info!(
                "number of empty ground tiles after 10 rounds: {}",
                num_empty_tiles
            );
        }

        let mut position_updates: HashMap<Coordinate, Option<Coordinate>> = HashMap::new();

        // first half
        for position in elf_positions.iter() {
            if has_neighbours(&elf_positions, &position, None) {
                for d in r..r + directions.len() {
                    let curr_d = directions[d % directions.len()];
                    if !has_neighbours(&elf_positions, &position, Some(&curr_d)) {
                        let next_pos = position.to_owned() + curr_d;
                        if let Some(curr_pos) = position_updates.get_mut(&next_pos) {
                            *curr_pos = None;
                        } else {
                            position_updates.insert(next_pos, Some(position.to_owned()));
                        }
                        break;
                    }
                }
            }
        }

        // second half
        let mut did_update = false;
        for (to, from) in position_updates.into_iter() {
            if let Some(position) = from {
                if elf_positions.remove(&position) && elf_positions.insert(to) {
                    did_update = true;
                    left_top.0 = cmp::min(left_top.0, *to.get_x());
                    left_top.1 = cmp::min(left_top.1, *to.get_y());
                    right_bottom.0 = cmp::max(right_bottom.0, *to.get_x());
                    right_bottom.1 = cmp::max(right_bottom.1, *to.get_y());
                } else {
                    bail!("tried to move elf that doesn't exist");
                }
            }
        }

        debug!("== End of Round {} ==", r);
        debug_region(&elf_positions, left_top, right_bottom);

        if !did_update {
            info!("first round where no elf moved: {}", r + 1);
            break;
        }
    }

    Ok(())
}

fn debug_region(
    elf_positions: &HashSet<Coordinate>,
    left_top: (i32, i32),
    right_bottom: (i32, i32),
) {
    let mut line = String::new();
    for x in left_top.0..=right_bottom.0 {
        for y in left_top.1..=right_bottom.1 {
            if elf_positions.contains(&Coordinate::of_tuple((x, y))) {
                line.push('#');
            } else {
                line.push('.');
            }
        }
        debug!("{}", line);
        line.clear();
    }
}

fn has_neighbours(
    elf_positions: &HashSet<Coordinate>,
    pos: &Coordinate,
    dir: Option<&Coordinate>,
) -> bool {
    (-1..=1)
        .cartesian_product(-1..=1)
        .filter_map(|(n_x, n_y)| {
            if let Some(dir) = dir {
                if !((*dir.get_x() == 0 && *dir.get_y() == n_y)
                    || *dir.get_y() == 0 && *dir.get_x() == n_x)
                {
                    return None;
                }
            } else if n_x == 0 && n_y == 0 {
                return None;
            }
            Some(pos.clone() + Coordinate::of_tuple((n_x, n_y)))
        })
        .fold(false, |acc, n| acc || elf_positions.contains(&n))
}

fn parse_input(input: String) -> Result<(HashSet<Coordinate>, (i32, i32), (i32, i32))> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut elf_positions = HashSet::new();
    let mut left_top = None;
    let mut right_bottom = None;

    let mut curr_row = 0;

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        for (curr_col, c) in line.trim().chars().enumerate() {
            if c == '#' {
                elf_positions.insert(Coordinate::of_tuple((curr_row, curr_col as i32)));
                if let Some((x, y)) = left_top.as_mut() {
                    *x = cmp::min(*x, curr_row);
                    *y = cmp::min(*y, curr_col as i32);
                } else {
                    left_top = Some((curr_row, curr_col as i32));
                }
                if let Some((x, y)) = right_bottom.as_mut() {
                    *x = cmp::max(*x, curr_row);
                    *y = cmp::max(*y, curr_col as i32);
                } else {
                    right_bottom = Some((curr_row, curr_col as i32));
                }
            }
        }

        line.clear();
        curr_row += 1
    }

    Ok((elf_positions, left_top.unwrap(), right_bottom.unwrap()))
}
