use anyhow::{bail, Context, Result};

use log::{debug, info};

use core::panic;
use std::io::{BufRead, BufReader};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fs::File,
    rc::Rc,
};

type Neighbour = Option<(u8, Rc<RefCell<Node>>)>; // (direction change, link)

struct Node {
    coord: (usize, usize),
    neighbours: Vec<Neighbour>,
}

impl Node {
    fn get_neighbour(&self, i: u8) -> Neighbour {
        self.neighbours[i as usize]
            .as_ref()
            .map(|(d, r)| (*d, Rc::clone(r)))
    }
}

pub fn solve(task: u8, input: String) -> Result<()> {
    let (mut curr_node, mut steps) = parse_input(input, task).context("failed to parse input")?;

    debug!(
        "starting at node ({}, {})",
        curr_node.borrow().coord.0,
        curr_node.borrow().coord.1
    );

    let mut curr_side = 0;
    while let Some((face_update, mut num_steps)) = steps.pop_back() {
        debug!("moving {} steps, facing {}", num_steps, face_update);
        curr_side = (curr_side as i8 + face_update).rem_euclid(4) as u8;
        while num_steps > 0 {
            let next_link = curr_node.borrow().get_neighbour(curr_side);
            if let Some((out_side, n)) = next_link {
                curr_side = out_side;
                curr_node = n;
                num_steps -= 1;
            } else {
                break;
            }
        }
        debug!(
            "moved to node ({}, {})",
            curr_node.borrow().coord.0,
            curr_node.borrow().coord.1
        );
    }

    let final_password =
        1000 * curr_node.borrow().coord.0 + 4 * curr_node.borrow().coord.1 + curr_side as usize;

    info!(
        "final password is: 1000 x {} + 4 x {} + {} = {}",
        curr_node.borrow().coord.0,
        curr_node.borrow().coord.1,
        curr_side,
        final_password
    );

    Ok(())
}

fn parse_input(input: String, task: u8) -> Result<(Rc<RefCell<Node>>, VecDeque<(i8, u32)>)> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut node_matrix = Vec::new();
    let mut row_start_i = Vec::new();
    let mut col_range_i: Vec<Option<(usize, usize)>> = Vec::new();
    let mut steps = VecDeque::new();

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        let start_index = line.len() - line.trim_start().len();
        (col_range_i.len()..line.trim_end().len()).for_each(|_| col_range_i.push(None));
        let mut row = Vec::with_capacity(line.trim_end().len() - 1);
        for (i, c) in line.trim_end().chars().enumerate() {
            match c {
                '.' => {
                    row.push(Some(Rc::new(RefCell::new(Node {
                        coord: (node_matrix.len() + 1, i + 1),
                        neighbours: vec![None; 4],
                    }))));
                    col_range_i[row.len() - 1] = Some(
                        col_range_i[row.len() - 1]
                            .map_or((node_matrix.len(), node_matrix.len()), |(s, e)| (s, e + 1)),
                    );
                }
                '#' => {
                    row.push(None);
                    col_range_i[row.len() - 1] = Some(
                        col_range_i[row.len() - 1]
                            .map_or((node_matrix.len(), node_matrix.len()), |(s, e)| (s, e + 1)),
                    );
                }
                _ => row.push(None),
            }
        }
        node_matrix.push(row);
        row_start_i.push(start_index);
        line.clear();
    }

    let start_node = match task {
        1 => connect_as_grid(node_matrix, row_start_i, col_range_i),
        2 => connect_as_cube(node_matrix, row_start_i),
        _ => bail!("task doesn't exist!"),
    };

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        debug!("parsing line {}", line);
        let mut orientation_change = 0;
        let mut curr_steps = String::new();
        line = line.trim().to_owned();
        for c in line.chars() {
            if c.is_ascii_digit() {
                curr_steps.push(c);
            } else {
                steps.push_front((orientation_change, curr_steps.parse::<u32>().unwrap()));
                curr_steps.clear();
                match c {
                    'L' => orientation_change = -1,
                    'R' => orientation_change = 1,
                    _ => bail!("unknown char {}", c),
                }
            }
        }
        steps.push_front((
            orientation_change,
            if curr_steps.is_empty() {
                0
            } else {
                curr_steps.parse::<u32>().unwrap()
            },
        ));
        line.clear();
    }

    Ok((start_node.unwrap(), steps))
}

fn connect_as_grid(
    node_matrix: Vec<Vec<Option<Rc<RefCell<Node>>>>>,
    row_start_i: Vec<usize>,
    col_range_i: Vec<Option<(usize, usize)>>,
) -> Option<Rc<RefCell<Node>>> {
    let mut curr_start = None;
    for x in 0..node_matrix.len() {
        let row = &node_matrix[x];
        let start_index = row_start_i[x];
        for y in 0..row.len() {
            if let Some(node) = &row[y] {
                curr_start = curr_start.or_else(|| Some(Rc::clone(&node)));
                let right_i = if y == row.len() - 1 {
                    start_index
                } else {
                    y + 1
                };
                if let Some(r_n) = &row[right_i] {
                    node.borrow_mut().neighbours[0] = Some((2, Rc::clone(&r_n)));
                    r_n.borrow_mut().neighbours[2] = Some((0, Rc::clone(&node)));
                }
                let down_i = if x == col_range_i[y].unwrap().1 {
                    col_range_i[y].unwrap().0
                } else {
                    x + 1
                };
                if let Some(d_n) = &node_matrix[down_i][y] {
                    node.borrow_mut().neighbours[1] = Some((3, Rc::clone(&d_n)));
                    d_n.borrow_mut().neighbours[3] = Some((1, Rc::clone(&node)));
                }
            }
        }
    }
    curr_start
}

fn connect_as_cube(
    node_matrix: Vec<Vec<Option<Rc<RefCell<Node>>>>>,
    row_start_i: Vec<usize>,
) -> Option<Rc<RefCell<Node>>> {
    let (face_connections, side_len) = fold_cube(&node_matrix, &row_start_i);

    let mut curr_start = None;
    for out_x in 0..node_matrix.len() {
        let row = &node_matrix[out_x];
        for out_y in 0..row.len() {
            if let Some(out_n) = &row[out_y] {
                curr_start = curr_start.or_else(|| Some(Rc::clone(&out_n)));
                for out_s in 0..4 {
                    let ((in_x, in_y), in_s) =
                        find_next_cell(&face_connections, side_len, (out_x, out_y), out_s);
                    if let Some(in_n) = &node_matrix[in_x][in_y] {
                        out_n.borrow_mut().neighbours[out_s as usize] =
                            Some(((in_s + 2) % 4, Rc::clone(&in_n)));
                        in_n.borrow_mut().neighbours[in_s as usize] =
                            Some(((out_s + 2) % 4, Rc::clone(&out_n)));
                    }
                }
            }
        }
    }
    curr_start
}

fn find_next_cell(
    face_connections: &HashMap<(usize, usize), VecDeque<((usize, usize), u8)>>,
    side_len: usize,
    (x, y): (usize, usize),
    out_s: u8,
) -> ((usize, usize), u8) {
    // TODO: still need to invert the offset if in_s == out_s
    if x % side_len == 0 && out_s == 3
        || x % side_len == 1 && out_s == 1
        || y % side_len == 0 && out_s == 2
        || y % side_len == 1 && out_s == 0
    {
        let from_corner = (x - x % side_len, y - y % side_len);
        let (to_corner, in_s) = face_connections[&from_corner][out_s as usize];
        let mut offset = if out_s % 2 == 0 {
            x % side_len
        } else {
            y % side_len
        };
        if in_s != (out_s + 2) % 4 {
            // invert offset
            offset = side_len - offset;
        }
        let (to_x, to_y) = match in_s {
            0 => (to_corner.0 + offset, to_corner.1 + side_len - 1),
            1 => (to_corner.0 + side_len - 1, to_corner.1 + offset),
            2 => (to_corner.0 + offset, to_corner.1),
            3 => (to_corner.0, to_corner.1 + offset),
            _ => panic!("trying to enter on unknown side"),
        };
        ((to_x, to_y), in_s)
    } else {
        // simple case: just stay on the face
        match out_s {
            0 => ((x, y + 1), 2),
            1 => ((x + 1, y), 3),
            2 => ((x, y - 1), 0),
            3 => ((x - 1, y), 1),
            _ => panic!("trying to leave from unknown side"),
        }
    }
}

fn fold_cube(
    node_matrix: &Vec<Vec<Option<Rc<RefCell<Node>>>>>,
    row_start_i: &Vec<usize>,
) -> (
    HashMap<(usize, usize), VecDeque<((usize, usize), u8)>>,
    usize,
) {
    let mut mapping = HashMap::new();
    let mut to_visit = VecDeque::new();

    let side_len = find_side_len(&node_matrix);

    let mut cube_lookup = HashMap::new();
    cube_lookup.insert('F', vec!['R', 'D', 'L', 'U']);
    cube_lookup.insert('L', vec!['F', 'D', 'B', 'U']);
    cube_lookup.insert('R', vec!['B', 'D', 'F', 'U']);
    cube_lookup.insert('B', vec!['L', 'D', 'R', 'U']);
    cube_lookup.insert('U', vec!['R', 'F', 'L', 'B']);
    cube_lookup.insert('D', vec!['R', 'B', 'L', 'F']);

    // (face_id, side) --> {(start_of_edge_0, edge_1_point), (end_of_edge_0, edge_1_point)}
    // ---> will allow applying offset from 'start' of edge_1 in direction of 'end'
    let mut edge_connections: HashMap<char, HashMap<char, ((usize, usize), u8)>> = HashMap::new();
    cube_lookup.keys().for_each(|f| {
        edge_connections.insert(*f, HashMap::new());
    });

    mapping.insert('F', ((0usize, row_start_i[0]), 0i8));
    to_visit.push_front('F');

    while let Some(curr_face) = to_visit.pop_back() {
        // discover faces in BFS manner
        let (curr_corner, curr_orientation) = mapping[&curr_face];

        debug!("==== face {} ====", curr_face);
        debug!("current state: {:?}", edge_connections);

        // current face should be connected by either 0 or 1 edge
        // if 1: check for folding angles and connect corresponding edges
        // for each newly added edge: check if another adjacent edge --> keep adding till 4 or done
        let mut edges_to_consider = Vec::new();
        if let Some((f, _)) = edge_connections[&curr_face].iter().next() {
            edges_to_consider.push(*f);
        }

        while let Some(connected_face) = edges_to_consider.pop() {
            debug!(
                "face {}: considering edge to face {}",
                curr_face, connected_face
            );
            let connected_face_i = cube_lookup[&curr_face]
                .iter()
                .position(|f| *f == connected_face)
                .unwrap();
            let neighbouring_faces = vec![
                cube_lookup[&curr_face][(connected_face_i + 1) % 4],
                cube_lookup[&curr_face][(connected_face_i + 3) % 4],
            ];
            for to_face in neighbouring_faces {
                if let Some((fold_c, _)) = edge_connections[&connected_face].get(&to_face) {
                    if edge_connections[&curr_face].get(&to_face).is_none() {
                        // glue edges
                        debug!("adding edge {} -- {}", curr_face, to_face);

                        let from_side =
                            get_s_for_face(&cube_lookup, &mapping, &curr_face, &to_face);
                        let to_side = get_s_for_face(&cube_lookup, &mapping, &to_face, &curr_face);

                        let from_to_corner =
                            find_op_corner(mapping[&to_face].0, *fold_c, side_len, to_side);

                        let from_edge_start = find_edge_start(curr_corner, side_len, from_side);
                        let to_edge_start = find_edge_start(mapping[&to_face].0, side_len, to_side);

                        let from_to_edge = (from_to_corner, to_side);
                        edge_connections
                            .get_mut(&curr_face)
                            .unwrap()
                            .insert(to_face, from_to_edge);

                        let to_from_corner = if to_edge_start == from_to_corner {
                            from_edge_start
                        } else {
                            find_op_corner(curr_corner, from_edge_start, side_len, from_side)
                        };
                        let to_from_edge = (to_from_corner, to_side);
                        edge_connections
                            .get_mut(&to_face)
                            .unwrap()
                            .insert(curr_face, to_from_edge);
                        edges_to_consider.push(to_face);
                    }
                }
            }
        }

        // from current face, check all 4 sides
        for (side, d) in vec![(0, 1), (1, 0), (0, -1), (-1, 0)].iter().enumerate() {
            // compute coordinate of potential next corner
            let next_corner =
                add_to_coord(curr_corner, (d.0 * side_len as i32, d.1 * side_len as i32));

            // check if corner exists
            if let Some((n_x, n_y)) = next_corner {
                if n_x < node_matrix.len()
                    && row_start_i[n_x] <= n_y
                    && n_y < node_matrix[n_x].len()
                {
                    // if it does, figure out which face it belongs to
                    let next_face = cube_lookup[&curr_face]
                        [(side as i8 - curr_orientation).rem_euclid(4) as usize];

                    // if we haven't seen this face yet, compute its orientation and push to stack
                    if !mapping.contains_key(&next_face) {
                        debug!("discovered face {}!", next_face);
                        let expected_side = cube_lookup[&next_face]
                            .iter()
                            .position(|c| *c == curr_face)
                            .unwrap();
                        let actual_side = (side + 2) % 4;
                        let orientation = (actual_side as i8 - expected_side as i8).rem_euclid(4);
                        mapping.insert(next_face, ((n_x, n_y), orientation));
                        to_visit.push_front(next_face);

                        // compute how the edges are connected
                        debug!("adding edge {} -- {}", curr_face, next_face);

                        let mut from_corner = find_edge_start(curr_corner, side_len, side as u8);
                        let mut to_corner = add_to_coord(from_corner, *d);
                        edge_connections
                            .get_mut(&curr_face)
                            .unwrap()
                            .insert(next_face, (to_corner.unwrap(), actual_side as u8));

                        from_corner = find_edge_start((n_x, n_y), side_len, actual_side as u8);
                        to_corner = add_to_coord(from_corner, (-d.0, -d.1));
                        edge_connections
                            .get_mut(&next_face)
                            .unwrap()
                            .insert(curr_face, (to_corner.unwrap(), side as u8));
                    }
                }
            }
        }
    }

    debug!("{:?}", edge_connections);
    // assert that all edges have been set
    debug_assert!(edge_connections
        .values()
        .fold(true, |acc, sides| acc && sides.len() == 4));

    panic!("edge connections actually work?!");

    // TODO: Figure out for each edge, which two corners are touching (are we inverted or not?)

    let mut cube_connections = HashMap::new();
    for (f, adj) in cube_lookup.iter() {
        let (corner, orientation) = mapping[&f];
        let mut neighbours = adj
            .iter()
            .map(|x| {
                let (to_corner, _) = mapping[x];
                let to_side = get_s_for_face(&cube_lookup, &mapping, x, f);
                (to_corner, to_side)
            })
            .collect::<VecDeque<_>>();
        neighbours.rotate_right(orientation as usize);
        cube_connections.insert(corner, neighbours);
    }
    (cube_connections, side_len)
}

fn get_s_for_face(
    cube_lookup: &HashMap<char, Vec<char>>,
    mapping: &HashMap<char, ((usize, usize), i8)>,
    from_face: &char,
    to_face: &char,
) -> u8 {
    let expected_in_s = cube_lookup[from_face]
        .iter()
        .position(|f| f == to_face)
        .unwrap();
    let actual_in_s = ((expected_in_s + mapping[from_face].1 as usize) % 4) as u8;
    actual_in_s
}

fn add_to_coord(coord: (usize, usize), diff: (i32, i32)) -> Option<(usize, usize)> {
    let res_0 = coord.0 as i32 + diff.0;
    let res_1 = coord.1 as i32 + diff.1;
    if res_0 < 0 || res_1 < 0 {
        None
    } else {
        Some((res_0 as usize, res_1 as usize))
    }
}

fn find_edge_start(face_coord: (usize, usize), side_len: usize, side: u8) -> (usize, usize) {
    match side {
        0 => (face_coord.0, face_coord.1 + side_len - 1),
        1 => (face_coord.0 + side_len - 1, face_coord.1),
        2 => face_coord,
        3 => face_coord,
        _ => panic!("illegal"),
    }
}

fn find_op_corner(
    face_coord: (usize, usize),
    curr_end: (usize, usize),
    side_len: usize,
    side: u8,
) -> (usize, usize) {
    let edge_start = find_edge_start(face_coord, side_len, side);
    match (side, edge_start == curr_end) {
        (0 | 2, true) => (curr_end.0 + side_len - 1, curr_end.1),
        (0 | 2, false) => (curr_end.0 + 1 - side_len, curr_end.1),
        (1 | 3, true) => (curr_end.0, curr_end.1 + side_len - 1),
        (1 | 3, false) => (curr_end.0, curr_end.1 + 1 - side_len),
        _ => panic!("unknown side value"),
    }
}

fn find_side_len(node_matrix: &Vec<Vec<Option<Rc<RefCell<Node>>>>>) -> usize {
    let mut curr_len = node_matrix[0].len();
    for i in 1..node_matrix.len() {
        curr_len = gcd::binary_usize(curr_len, node_matrix[i].len());
    }
    curr_len
}
