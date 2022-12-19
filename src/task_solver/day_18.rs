use anyhow::{bail, Context, Result};

use log::info;

use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

type Coord = (i32, i32, i32);

pub fn solve(task: u8, input: String) -> Result<()> {
    let droplet = parse_input(input).context("failed to parse input")?;

    match task {
        1 => {
            let num_faces = get_num_faces(&droplet);

            info!(
                "The surface area of the scanned lava dropplet is {}",
                num_faces
            );
        }
        2 => {
            let num_outer_faces = get_num_outer_faces(&droplet);

            info!(
                "The exterior surface area of the scanned lava dropplet is {}",
                num_outer_faces
            );
        }
        _ => bail!("task doesn't exist!"),
    }

    Ok(())
}

fn parse_input(input: String) -> Result<HashSet<Coord>> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut droplet = HashSet::new();

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        let cube: Vec<i32> = line
            .trim()
            .split(',')
            .map(|s| {
                s.parse::<i32>()
                    .expect(&format!("can't parse u32 from input {}", s))
            })
            .collect();

        let cube_coord = (cube[0], cube[1], cube[2]);
        droplet.insert(cube_coord);

        line.clear();
    }

    Ok(droplet)
}

fn get_num_faces(droplet: &HashSet<Coord>) -> u32 {
    droplet.iter().fold(0u32, |acc, cube| {
        acc + 6 - get_num_adjacent_cubes(droplet, cube)
    })
}

fn get_num_outer_faces(droplet: &HashSet<Coord>) -> u32 {
    // find starting point: cube with max coordinates in a direction
    let outer_cube = droplet.iter().fold(&(0i32, 0i32, 0i32), |curr_max, cube| {
        if cube.0 > curr_max.0 {
            // just get the cube with max x coordinate
            cube
        } else {
            curr_max
        }
    });

    // walk along the outside of the droplet from there --> to_visit and visited set
    let mut num_outer_faces = 0u32;
    let mut visited = HashSet::new();
    let mut to_visit = HashSet::new();
    let starting_cube = (outer_cube.0 + 1, outer_cube.1, outer_cube.2);
    to_visit.insert(starting_cube);

    while let Some(cube) = pop_from_set(&mut to_visit) {
        let num_adjacent_faces = get_num_adjacent_cubes(droplet, &cube);
        let mut neighbours = get_neighbours(&cube);
        to_visit.extend(if num_adjacent_faces == 0 {
            neighbours.retain(|c| {
                !visited.contains(c)
                    && !droplet.contains(c)
                    && get_num_adjacent_cubes(droplet, c) != 0
            });
            neighbours
        } else {
            neighbours.retain(|c| !visited.contains(c) && !droplet.contains(c));
            neighbours
        });
        visited.insert(cube);
        num_outer_faces += num_adjacent_faces;
    }

    num_outer_faces
}

fn pop_from_set(set: &mut HashSet<Coord>) -> Option<Coord> {
    if let Some(elem) = set.iter().next().cloned() {
        set.remove(&elem);
        Some(elem)
    } else {
        None
    }
}

fn get_neighbours(cube: &Coord) -> HashSet<Coord> {
    let mut neighbours = HashSet::new();
    neighbours.insert((cube.0 - 1, cube.1, cube.2));
    neighbours.insert((cube.0 + 1, cube.1, cube.2));
    neighbours.insert((cube.0, cube.1 - 1, cube.2));
    neighbours.insert((cube.0, cube.1 + 1, cube.2));
    neighbours.insert((cube.0, cube.1, cube.2 - 1));
    neighbours.insert((cube.0, cube.1, cube.2 + 1));
    neighbours
}

fn get_num_adjacent_cubes(droplet: &HashSet<Coord>, cube: &Coord) -> u32 {
    get_neighbours(cube).iter().fold(0u32, |mut acc, c| {
        if droplet.contains(c) {
            acc += 1;
        }
        acc
    })
}
