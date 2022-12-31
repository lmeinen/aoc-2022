use anyhow::{bail, Context, Result};

use log::{debug, info};
use num::integer::gcd;

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::vec;
use std::{cell::RefCell, collections::VecDeque, fs::File, rc::Rc};

type NodeRef = Option<Rc<RefCell<Node>>>;
type Grid = Vec<Vec<NodeRef>>;

#[derive(PartialEq)]
struct Node {
    coord: (usize, usize),
    neighbours: Vec<NodeRef>,
}

impl Node {
    fn get_neighbour(&self, i: u8) -> Option<(Rc<RefCell<Node>>, u8)> {
        self.neighbours[i as usize].as_ref().map(|r| {
            let out_face = (**r)
                .borrow()
                .neighbours
                .iter()
                .enumerate()
                .find_map(|(in_face, n)| {
                    if n.is_some() && (**(n.as_ref().unwrap())).borrow().coord == self.coord {
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

    let start_node = find_start(&grid).expect("failed to find starting node");
    match task {
        1 => connect(grid, &obstacles, wrap_grid),
        2 => {
            let wrap = build_cube(&grid, (*start_node).borrow().coord);
            connect(grid, &obstacles, wrap)
        }
        _ => bail!("task doesn't exist!"),
    }

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

fn connect<W>(grid: Grid, obstacles: &HashSet<(usize, usize)>, wrap: W)
where
    W: Fn(&Grid, i32, i32, usize, &(i32, i32)) -> NodeRef,
{
    for x in 0..grid.len() {
        let row = &grid[x];
        for y in 0..row.len() {
            if let Some(node) = &row[y] {
                for (i, d) in vec![(0, 1), (1, 0), (0, -1), (-1, 0)].iter().enumerate() {
                    let neighbour = get_node(&grid, (x as i32) + d.0, (y as i32) + d.1)
                        .unwrap_or_else(|| {
                            wrap(&grid, x as i32, y as i32, i, d).expect("wrapping function failed")
                        });

                    if !obstacles.contains(&(*neighbour).borrow().coord) {
                        node.borrow_mut().neighbours[i] = Some(neighbour);
                    }
                }
            }
        }
    }
}

fn get_node(grid: &Grid, x: i32, y: i32) -> NodeRef {
    if !((0..grid.len() as i32).contains(&x) && (0..grid[x as usize].len() as i32).contains(&y)) {
        None
    } else {
        grid[x as usize][y as usize].as_ref().map(|n| Rc::clone(n))
    }
}

fn find_start(grid: &Grid) -> NodeRef {
    for i in 0..grid[0].len() {
        if let Some(n) = &grid[0][i] {
            return Some(Rc::clone(n));
        }
    }
    None
}

fn wrap_grid(grid: &Grid, mut x: i32, mut y: i32, _: usize, d: &(i32, i32)) -> NodeRef {
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

fn build_cube(
    grid: &Grid,
    (x, y): (usize, usize),
) -> Box<dyn Fn(&Grid, i32, i32, usize, &(i32, i32)) -> NodeRef> {
    let (side_len, faces) = walk_edges(grid, x, y);
    Box::new(move |grid, x, y, d, _| {
        let offset = (x as usize % side_len, y as usize % side_len);
        let face = ((x as usize - offset.0), (y as usize - offset.1));
        // find neighbour
        let neighbour = faces[&face][d];
        // find the corresponding direction at neighbour
        let d_n = faces[&neighbour].iter().position(|f| f == &face).unwrap();
        // consider which face we're walking towards on the edge, replicate the same direction on neighbour's edge
        let towards_face = if d == 1 || d == 2 {
            faces[&face][(d + 3) % 4]
        } else {
            faces[&face][(d + 1) % 4]
        };
        let walk_clockwise = towards_face == faces[&neighbour][(d_n + 1) % 4];
        let dest_coord = compute_dest_coord(side_len, neighbour, offset, d, d_n, walk_clockwise);
        get_node(grid, dest_coord.0 as i32, dest_coord.1 as i32)
    })
}

fn compute_dest_coord(
    side_len: usize,
    neighbour: (usize, usize),
    offset: (usize, usize),
    from_d: usize,
    to_d: usize,
    walk_clockwise: bool,
) -> (usize, usize) {
    let neighbour_vec = vec![neighbour.0, neighbour.1];
    let offset = if from_d % 2 == to_d % 2 {
        vec![offset.0, offset.1]
    } else {
        vec![offset.1, offset.0]
    };
    let mut dest = vec![0; 2];
    dest[to_d % 2] = neighbour_vec[to_d % 2]
        + if (to_d % 3 == 0 && walk_clockwise) || (to_d % 3 != 0 && !walk_clockwise) {
            offset[to_d % 2]
        } else {
            (side_len - 1) - offset[to_d % 2]
        };
    dest[(to_d + 1) % 2] = neighbour_vec[(to_d + 1) % 2] + if to_d < 2 { side_len - 1 } else { 0 };
    (dest[0], dest[1])
}

fn walk_edges(
    grid: &Grid,
    x: usize,
    y: usize,
) -> (usize, HashMap<(usize, usize), Vec<(usize, usize)>>) {
    // 1. find edge length
    let mut side_len = grid.len();
    for line in grid.iter() {
        side_len = gcd(side_len, line.len());
    }

    // 2. label first face
    let mut labels = Vec::with_capacity(6);
    labels.push(vec![1, 2, 3, 4]);
    labels.push(vec![5, 2, 0, 4]);
    labels.push(vec![1, 5, 3, 0]);
    labels.push(vec![0, 2, 5, 4]);
    labels.push(vec![3, 5, 1, 0]);
    labels.push(vec![3, 2, 1, 4]);

    // 3. walk along edges - label each discovered face and decide orientation
    let mut to_visit = VecDeque::with_capacity(6);
    let mut faces = HashMap::new();
    to_visit.push_back((0, (x, y)));
    while let Some((face, coord)) = to_visit.pop_front() {
        let mut new_faces = Vec::new();
        let mut orientation = 0;
        // 3a. check for neighbouring faces
        for (i, d) in vec![(0, 1), (1, 0), (0, -1), (-1, 0)].iter().enumerate() {
            if let Some(neighbour) = get_node(
                &grid,
                (coord.0 as i32) + d.0 * side_len as i32,
                (coord.1 as i32) + d.1 * side_len as i32,
            ) {
                // 3b. check if those faces have been discovered yet:
                if let Some(neighbour_face) = faces.get(&(*neighbour).borrow().coord) {
                    // - if yes, check label to decide on own orientation
                    orientation = (labels[face]
                        .iter()
                        .position(|f| f == neighbour_face)
                        .expect("these two faces shouldn't be neighbouring!")
                        as i32
                        - i as i32)
                        .rem_euclid(4) as usize;
                } else {
                    // - if not, add to list of coordinates to be added to to_visit (wait until all directions visited before deciding which face it is)
                    new_faces.push((i, (*neighbour).borrow().coord));
                }
            }
        }

        // 3c. iterate list of newly discovered faces and label them according to current orientation, then add to to_visit
        for (i, neighbour_coord) in new_faces.into_iter() {
            to_visit.push_back((labels[face][(i + orientation) % 4], neighbour_coord));
        }

        labels[face].rotate_left(orientation);
        faces.insert(coord, face);
    }

    let faces = faces
        .into_iter()
        .map(|(coord, face)| (face, coord))
        .collect::<HashMap<usize, (usize, usize)>>();

    (
        side_len,
        labels
            .into_iter()
            .enumerate()
            .map(|(face, neighbours)| {
                (
                    faces[&face],
                    neighbours
                        .into_iter()
                        .map(|neighbour| faces[&neighbour])
                        .collect(),
                )
            })
            .collect(),
    )
}
