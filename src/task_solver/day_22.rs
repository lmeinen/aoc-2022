use anyhow::{bail, Context, Result};

use log::{debug, info};

use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::{cell::RefCell, collections::VecDeque, fs::File, rc::Rc};

type NodeRef = Option<Rc<RefCell<Node>>>;
type Grid = Vec<Vec<NodeRef>>;

struct Node {
    coord: (usize, usize),
    neighbours: Vec<NodeRef>,
}

impl Node {
    fn get_neighbour(&self, i: u8) -> Option<(Rc<RefCell<Node>>, u8)> {
        self.neighbours[i as usize].as_ref().map(|r| {
            let out_face = r
                .borrow()
                .neighbours
                .iter()
                .enumerate()
                .find_map(|(in_face, n)| {
                    if n.is_some() && n.as_ref().unwrap().borrow().coord == self.coord {
                        Some((in_face + 2) % 4)
                    } else {
                        None
                    }
                })
                .expect("couldn't find incoming face on neighbour");
            (Rc::clone(r), out_face as u8)
        })
    }
}

pub fn solve(task: u8, input: String) -> Result<()> {
    let (mut node, mut steps) = parse_input(input, task).context("failed to parse input")?;

    debug!(
        "starting at node ({}, {})",
        (*node).borrow().coord.0,
        (*node).borrow().coord.1
    );

    let mut face = 0;
    while let Some((orientation_change, mut num_steps)) = steps.pop_back() {
        debug!("moving {} steps, facing {}", num_steps, orientation_change);
        face = (face + orientation_change) % 4;
        while num_steps > 0 {
            let next_position = (*node).borrow().get_neighbour(face);
            if let Some((next, out_face)) = next_position {
                face = out_face;
                node = next;
                num_steps -= 1;
            } else {
                break;
            }
        }
        debug!(
            "moved to node ({}, {})",
            (*node).borrow().coord.0,
            (*node).borrow().coord.1
        );
    }

    let final_password =
        1000 * ((*node).borrow().coord.0 + 1) + 4 * ((*node).borrow().coord.1 + 1) + face as usize;

    info!(
        "final password is: 1000 x {} + 4 x {} + {} = {}",
        (*node).borrow().coord.0,
        (*node).borrow().coord.1,
        face,
        final_password
    );

    Ok(())
}

fn parse_input(input: String, task: u8) -> Result<(Rc<RefCell<Node>>, VecDeque<(u8, u32)>)> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut grid = Vec::new();
    let mut obstacles = HashSet::new();
    let mut path = VecDeque::new();

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        let mut row = Vec::with_capacity(line.trim_end().len() - 1);
        for (i, c) in line.trim_end().chars().enumerate() {
            let coord = (grid.len(), i);
            match c {
                '.' => {
                    row.push(Some(Rc::new(RefCell::new(Node {
                        coord,
                        neighbours: vec![None; 4],
                    }))));
                }
                '#' => {
                    row.push(Some(Rc::new(RefCell::new(Node {
                        coord,
                        neighbours: vec![None; 4],
                    }))));
                    obstacles.insert(coord);
                }
                _ => row.push(None),
            }
        }
        grid.push(row);
        line.clear();
    }
    line.clear();

    let start_node = match task {
        1 => connect(grid, &obstacles, &Box::new(wrap_grid)),
        2 => todo!("not implemented"),
        _ => bail!("task doesn't exist!"),
    }
    .expect("failed to find starting node");

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        debug!("parsing line {}", line);
        let mut orientation_change = 0u8;
        let mut curr_steps = String::new();
        for c in line.chars() {
            if c.is_ascii_digit() {
                curr_steps.push(c);
            } else {
                path.push_front((orientation_change, curr_steps.parse::<u32>().unwrap()));
                curr_steps.clear();
                match c {
                    'L' => orientation_change = 3,
                    'R' => orientation_change = 1,
                    _ => (),
                }
            }
        }
        line.clear();
    }

    Ok((start_node, path))
}

fn connect(
    grid: Grid,
    obstacles: &HashSet<(usize, usize)>,
    wrap: &Box<fn(&Grid, i32, i32, &(i32, i32)) -> NodeRef>,
) -> NodeRef {
    let mut curr_start = None;
    for x in 0..grid.len() {
        let row = &grid[x];
        for y in 0..row.len() {
            if let Some(node) = &row[y] {
                curr_start = curr_start.or_else(|| {
                    if !obstacles.contains(&(**node).borrow().coord) {
                        Some(Rc::clone(node))
                    } else {
                        None
                    }
                });
                for (i, d) in vec![(0, 1), (1, 0), (0, -1), (-1, 0)].iter().enumerate() {
                    let neighbour = get_node(&grid, (x as i32) + d.0, (y as i32) + d.1)
                        .unwrap_or_else(|| {
                            wrap(&grid, x as i32, y as i32, d).expect("wrapping function failed")
                        });

                    if !obstacles.contains(&(*neighbour).borrow().coord) {
                        node.borrow_mut().neighbours[i] = Some(neighbour);
                    }
                }
            }
        }
    }
    curr_start
}

fn get_node(grid: &Grid, x: i32, y: i32) -> NodeRef {
    if !((0..grid.len() as i32).contains(&x) && (0..grid[x as usize].len() as i32).contains(&y)) {
        None
    } else {
        grid[x as usize][y as usize].as_ref().map(|n| Rc::clone(n))
    }
}

fn wrap_grid(grid: &Grid, mut x: i32, mut y: i32, d: &(i32, i32)) -> NodeRef {
    let (n_x, n_y) = loop {
        if let Some(_) = get_node(grid, x - d.0, y - d.1) {
            x -= d.0;
            y -= d.1;
        } else {
            break (x, y);
        }
    };
    get_node(grid, n_x, n_y)
}
