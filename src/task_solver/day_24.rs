use anyhow::{bail, Context, Result};

use log::{debug, info};

use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

use super::util::Point;

type Coordinate = Point<u32>;
type Blizzard = Option<i8>;
type BlizzardList = Vec<Vec<Blizzard>>;

pub fn solve(_task: u8, input: String) -> Result<()> {
    let (hor_blizzards, vert_blizzards, start, end) =
        parse_input(input).context("failed to parse input")?;

    let cycles = num::integer::lcm(hor_blizzards.len() - 2, vert_blizzards.len() - 2) as u32;
    let mut to_visit = HashSet::new(); // LIFO queue to simulate BFS for Dijkstra
    let mut to_visit_next = HashSet::new();
    let mut visited = HashSet::new();
    let mut found_start = false;
    let mut found_end = false;

    to_visit.insert(start);
    'outer: for s in 0.. {
        'inner: for p in to_visit.drain() {
            debug!("field {:?} in round {}", p, s);
            for n in find_moves(&hor_blizzards, &vert_blizzards, &start, &end, &p, s).into_iter() {
                if !visited.contains(&(n, (s + 1) % cycles)) {
                    if n == end {
                        if !found_end {
                            found_end = true;
                            info!("shortest path to end has length: {}", s + 1);
                            visited.clear();
                            to_visit_next.clear();
                            to_visit_next.insert(end);
                            break 'inner;
                        } else if found_start {
                            info!(
                                "shortest path there, and back, and there again has length: {}",
                                s + 1
                            );
                            break 'outer;
                        }
                    } else if n == start && !found_start && found_end {
                        found_start = true;
                        info!("made it back to start");
                        visited.clear();
                        to_visit_next.clear();
                        to_visit_next.insert(start);
                        break 'inner;
                    }
                    to_visit_next.insert(n);
                }
            }
            visited.insert((p, s % cycles));
        }
        let tmp = to_visit;
        to_visit = to_visit_next;
        to_visit_next = tmp;
    }

    Ok(())
}

fn find_moves(
    hor_blizzards: &BlizzardList,
    vert_blizzards: &BlizzardList,
    start: &Coordinate,
    end: &Coordinate,
    p: &Coordinate,
    round: u32,
) -> Vec<Coordinate> {
    let dir = if p == start {
        vec![(0, 0), (1, 0)]
    } else if p == end {
        vec![(0, 0), (-1, 0)]
    } else {
        vec![(0, 0), (-1, 0), (0, 1), (1, 0), (0, -1)]
    };
    dir.into_iter()
        .filter_map(|d| {
            let n = neighbour(p, d);
            if check_field(hor_blizzards, vert_blizzards, &n, (round + 1) as i32) {
                Some(n)
            } else {
                None
            }
        })
        .collect()
}

fn neighbour(p: &Coordinate, d: (i32, i32)) -> Coordinate {
    Coordinate::of_tuple((
        (*p.get_x() as i32 + d.0) as u32,
        (*p.get_y() as i32 + d.1) as u32,
    ))
}

fn check_field(
    hor_blizzards: &BlizzardList,
    vert_blizzards: &BlizzardList,
    p: &Coordinate,
    round: i32,
) -> bool {
    let x = *p.get_x() as usize;
    let y = *p.get_y() as usize;
    let row = &hor_blizzards[x];
    let col = &vert_blizzards[y];
    if let Some(0) = row[y] {
        false
    } else if let Some(0) = col[x] {
        false
    } else if !(x == 0 || x == hor_blizzards.len() - 1) {
        if let Some(-1) = get_blizz(row, y, round) {
            false
        } else if let Some(1) = get_blizz(row, y, -round) {
            false
        } else if let Some(-1) = get_blizz(col, x, round) {
            false
        } else if let Some(1) = get_blizz(col, x, -round) {
            false
        } else {
            true
        }
    } else {
        true
    }
}

fn get_blizz(blizzard_list: &Vec<Blizzard>, p: usize, round: i32) -> Blizzard {
    blizzard_list[1 + (p as i32 - 1 + round).rem_euclid(blizzard_list.len() as i32 - 2) as usize]
}

fn parse_input(input: String) -> Result<(BlizzardList, BlizzardList, Coordinate, Coordinate)> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut hor_blizzards = Vec::new();
    let mut vert_blizzards = Vec::new();
    let mut start = None;

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        let mut row = Vec::new();
        for (curr_col, c) in line.trim().chars().enumerate() {
            let col = if let Some(v_blizzard) = vert_blizzards.get_mut(curr_col) {
                v_blizzard
            } else {
                vert_blizzards.push(vec![]);
                vert_blizzards.get_mut(curr_col).unwrap()
            };

            match c {
                '#' => {
                    row.push(Some(0));
                    col.push(Some(0));
                }
                '>' => {
                    row.push(Some(1));
                    col.push(None);
                }
                '<' => {
                    row.push(Some(-1));
                    col.push(None);
                }
                '^' => {
                    row.push(None);
                    col.push(Some(-1));
                }
                'v' => {
                    row.push(None);
                    col.push(Some(1));
                }
                '.' => {
                    if hor_blizzards.is_empty() {
                        start = Some(Coordinate::of_tuple((0, curr_col as u32)));
                    }
                    row.push(None);
                    col.push(None);
                }
                _ => bail!("unknown character: '{}'", c),
            }
        }
        if row.len() > 1 {
            hor_blizzards.push(row);
        }
        line.clear();
    }

    let end = Coordinate::of_tuple((
        hor_blizzards.len() as u32 - 1,
        hor_blizzards[hor_blizzards.len() - 1]
            .iter()
            .enumerate()
            .find(|(_, f)| f.is_none())
            .unwrap()
            .0 as u32,
    ));

    Ok((hor_blizzards, vert_blizzards, start.unwrap(), end))
}
