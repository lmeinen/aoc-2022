use anyhow::{anyhow, Context, Result};
use core::panic;
use log::{debug, info};
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader},
};

pub fn solve(_task: u8, input: String) -> Result<()> {
    // instantiate parser
    let parser = TreeParser::init(input).context("Failed to instantiate parser")?;

    match _task {
        1 => solve_1(parser),
        2 => solve_2(parser),
        _ => Err(anyhow!("task doesn't exist!")),
    }
}

fn solve_1(parser: TreeParser) -> Result<()> {
    let mut tree_grid = Vec::new();
    let mut row_viewpoints = Vec::new();
    let mut col_viewpoints = Vec::new();
    for (x, y, tree_height) in parser {
        debug!("tree at ({},{}) has height {}", x, y, tree_height);
        if row_viewpoints.len() < (x + 1) as usize {
            tree_grid.push(Vec::new());
            row_viewpoints.push((Viewpoint::init(true), Viewpoint::init(false)));
        }
        tree_grid[x as usize].push(tree_height);
        if col_viewpoints.len() < (y + 1) as usize {
            col_viewpoints.push((Viewpoint::init(true), Viewpoint::init(false)));
        }
        row_viewpoints[x as usize].0.add_tree(y, tree_height);
        row_viewpoints[x as usize].1.add_tree(98 - y, tree_height);
        col_viewpoints[y as usize].0.add_tree(x, tree_height);
        col_viewpoints[y as usize].1.add_tree(98 - x, tree_height);
    }

    let mut visible = 0u32;
    for (x, tree_row) in tree_grid.iter().enumerate() {
        let (row_start, row_end) = &row_viewpoints[x];
        for (y, tree_height) in tree_row.iter().enumerate() {
            let (col_start, col_end) = &col_viewpoints[y];
            if row_start.sees_tree(y, tree_height)
                || row_end.sees_tree(y, tree_height)
                || col_start.sees_tree(x, tree_height)
                || col_end.sees_tree(x, tree_height)
            {
                visible += 1;
            }
        }
    }

    info!("number of visible trees: {}", visible);

    Ok(())
}

fn solve_2(parser: TreeParser) -> Result<()> {
    let mut tree_grid = Vec::new();
    for (x, y, tree_height) in parser {
        debug!("tree at ({},{}) has height {}", x, y, tree_height);
        if tree_grid.len() < (x + 1) as usize {
            tree_grid.push(Vec::new());
        }
        tree_grid[x as usize].push(TreeView::init(tree_height as usize));
    }

    let tree_len = tree_grid.len();

    let mut curr_views_top: Vec<Vec<u32>> = (0..tree_len)
        .map(|_| vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        .collect(); // if I'm a tree of this height, how many trees can I see?
    let mut prev_height_top: Vec<usize> = (0..tree_len).map(|_| 0).collect();

    for x in 0..tree_len {
        let mut curr_views_left = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // if I'm a tree of this height, how many trees can I see?
        let curr_row = tree_grid.get_mut(x).unwrap();
        let mut prev_height_left = curr_row.get(0).unwrap().height;

        for y in 0..tree_len {
            if y != 0 {
                curr_views_left = curr_views_left
                    .iter_mut()
                    .enumerate()
                    .map(|(height, num_trees)| {
                        if height <= prev_height_left {
                            1
                        } else {
                            *num_trees + 1
                        }
                    })
                    .collect();
            }
            if x != 0 {
                curr_views_top[y] = curr_views_top[y]
                    .iter_mut()
                    .enumerate()
                    .map(|(height, num_trees)| {
                        if height <= prev_height_top[y] {
                            1
                        } else {
                            *num_trees + 1
                        }
                    })
                    .collect();
            }
            let curr_tree = curr_row.get_mut(y).unwrap();
            curr_tree.up = curr_views_top[y][curr_tree.height];
            curr_tree.left = curr_views_left[curr_tree.height];
            prev_height_top[y] = curr_tree.height;
            prev_height_left = curr_tree.height;
        }
    }

    let mut curr_views_bottom: Vec<Vec<u32>> = (0..tree_len)
        .map(|_| vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        .collect(); // if I'm a tree of this height, how many trees can I see?
    let mut prev_height_bottom: Vec<usize> = (0..tree_len).map(|_| 0).collect();

    for x in (0..tree_len).rev() {
        let mut curr_views_right = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // if I'm a tree of this height, how many trees can I see?
        let curr_row = tree_grid.get_mut(x).unwrap();
        let mut prev_height_right = curr_row.get(tree_len - 1).unwrap().height;

        for y in (0..tree_len).rev() {
            if y != tree_len - 1 {
                curr_views_right = curr_views_right
                    .iter_mut()
                    .enumerate()
                    .map(|(height, num_trees)| {
                        if height <= prev_height_right {
                            1
                        } else {
                            *num_trees + 1
                        }
                    })
                    .collect();
            }
            if x != tree_len - 1 {
                curr_views_bottom[y] = curr_views_bottom[y]
                    .iter_mut()
                    .enumerate()
                    .map(|(height, num_trees)| {
                        if height <= prev_height_bottom[y] {
                            1
                        } else {
                            *num_trees + 1
                        }
                    })
                    .collect();
            }
            let curr_tree = curr_row.get_mut(y).unwrap();
            curr_tree.down = curr_views_bottom[y][curr_tree.height];
            curr_tree.right = curr_views_right[curr_tree.height];
            prev_height_bottom[y] = curr_tree.height;
            prev_height_right = curr_tree.height;
        }
    }

    let mut max_score = 0;
    for tree_row in tree_grid.iter() {
        for tree in tree_row.iter() {
            max_score = std::cmp::max(max_score, tree.get_score());
        }
    }

    info!("top score: {}", max_score);

    Ok(())
}

struct TreeView {
    height: usize,
    left: u32,
    right: u32,
    up: u32,
    down: u32,
}

impl TreeView {
    fn init(height: usize) -> Self {
        TreeView {
            height,
            left: 0,
            right: 0,
            up: 0,
            down: 0,
        }
    }

    fn get_score(&self) -> u32 {
        let score = self.left * self.right * self.up * self.down;
        debug!("tree of height {} with score {}", self.height, score);
        score
    }
}

#[derive(Debug)]
struct Viewpoint {
    tree_visibilities: Vec<i32>, // each index represents the distance in hops until which we can see the corresponding height
    is_start: bool,
}

impl Viewpoint {
    fn init(is_start: bool) -> Self {
        Viewpoint {
            tree_visibilities: vec![-1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
            is_start,
        }
    }

    fn add_tree(&mut self, num_hops: u32, tree_height: u32) {
        if self.tree_visibilities[tree_height as usize] == -1
            || self.tree_visibilities[tree_height as usize] > num_hops as i32
        {
            self.tree_visibilities[tree_height as usize] = num_hops as i32;
        }
    }

    fn sees_tree(&self, index: usize, height: &u32) -> bool {
        let tree_visible = self.tree_visibilities[*height as usize..]
            .iter()
            .all(|hops| {
                (self.is_start && hops >= &(index as i32))
                    || (!self.is_start && hops >= &(98 - index as i32))
                    || *hops == -1
            });
        debug!(
            "checking if tree {} with height {} is visible for: {:?} --> {}",
            index, height, self, tree_visible
        );
        tree_visible
    }
}

struct TreeParser {
    in_reader: BufReader<File>,
    char_queue: VecDeque<char>,
    curr_x: u32,
    curr_y: u32,
}

impl TreeParser {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let in_reader = BufReader::new(in_file);

        Ok(TreeParser {
            in_reader,
            char_queue: VecDeque::new(),
            curr_x: 0,
            curr_y: 0,
        })
    }

    fn get_next_line(&mut self) -> Option<()> {
        let mut line = String::new();
        let bytes_read = self
            .in_reader
            .read_line(&mut line)
            .expect("Failed to read line in input file");
        if bytes_read == 0 || line == "\n" {
            return None; // EOF
        }
        self.char_queue = line.chars().collect();
        self.curr_y = 0;
        Some(())
    }
}

impl Iterator for TreeParser {
    type Item = (u32, u32, u32); // x, y, height

    fn next(&mut self) -> Option<Self::Item> {
        let next_char_option = self.char_queue.pop_front();
        let next_char = if next_char_option.is_none() {
            if self.get_next_line().is_some() {
                self.char_queue.pop_front().unwrap()
            } else {
                return None;
            }
        } else if next_char_option.unwrap() == '\n' {
            self.curr_x += 1;
            if self.get_next_line().is_some() {
                self.char_queue.pop_front().unwrap()
            } else {
                return None;
            }
        } else {
            self.curr_y += 1;
            next_char_option.unwrap()
        };

        if let Some(height) = next_char.to_digit(10) {
            Some((self.curr_x, self.curr_y, height))
        } else {
            panic!("couldn't parse height from character: {}", next_char)
        }
    }
}
