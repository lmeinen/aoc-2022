use anyhow::{bail, Context, Result};

use log::{debug, info};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    vec,
};

pub fn solve(_task: u8, input: String) -> Result<()> {
    let mut tetris_tower = TetrisTower::init(input).context("failed to instantiate parser")?;

    let n_iterations = match _task {
        1 => 2022,
        2 => 1000000000000,
        _ => bail!("task doesn't exist!"),
    };

    let (repeating_state, (i_0, h_0), (i_1, h_1)) =
        find_repeating_sequence(&mut tetris_tower).context("failed to find repeating sequence")?;

    let height = if n_iterations < i_0 {
        tetris_tower.reset_state();
        let (_, h) = tetris_tower.nth(n_iterations - 1).unwrap();
        h + tetris_tower.tower.len()
    } else {
        let mut rem_iterations = n_iterations - i_0 - 1; // this many rocks still need to be dropped
        let curr_height = h_0 + rem_iterations / (i_1 - i_0) * (h_1 - h_0);
        rem_iterations = rem_iterations % (i_1 - i_0);
        tetris_tower.reset_state_to(repeating_state, curr_height);
        let (_, h) = tetris_tower.nth(rem_iterations - 1).unwrap();
        h + tetris_tower.tower.len()
    };

    info!(
        "tower is {} units tall after {} rocks have stopped falling",
        height, n_iterations
    );

    Ok(())
}

fn find_repeating_sequence(
    tetris_tower: &mut TetrisTower,
) -> Result<(State, (usize, usize), (usize, usize))> {
    let mut seen_states = HashMap::new();
    for (i, (state, h)) in tetris_tower.enumerate() {
        debug!("i: {}, h: {}", i, h);
        if !seen_states.contains_key(&state) {
            seen_states.insert(state, (i, h));
        } else {
            info!("found repeating state after {} iterations", i);
            let (i_prev, h_prev) = seen_states.get(&state).unwrap();
            return Ok((state, (*i_prev, *h_prev), (i, h)));
        }
    }
    bail!("tetris tower iteration terminated before we could find a repeating sequence!");
}

type State = (Vec<u8>, usize, usize);

#[derive(Debug)]
enum Jet {
    LEFT,
    RIGHT,
}

enum TetrisBlock {
    HOR,
    PLUS,
    MIRRORED,
    VERT,
    SQUARE,
}

struct TetrisTower {
    /// list of jet directions to be iterated in falling rock simulation
    jet_pattern: Vec<Jet>,
    /// list of rock patterns to be iterated over
    rock_pattern: Vec<TetrisBlock>,
    /// list of bits - indicating height in units and how the tower is currently filled
    tower: Vec<u8>,
    /// number of fallen rocks - used to index jet_pattern and decide next rock to drop
    rock_no: usize,
    /// number of passed jets - used to index jet_pattern and decide next jet to use
    jet_no: usize,
    /// total tower height
    height: usize,
}

impl TetrisTower {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let mut in_reader = BufReader::new(in_file);
        let mut line = String::new();

        let mut jet_pattern = Vec::new();
        let rock_pattern = vec![
            TetrisBlock::HOR,
            TetrisBlock::PLUS,
            TetrisBlock::MIRRORED,
            TetrisBlock::VERT,
            TetrisBlock::SQUARE,
        ];
        let tower = Vec::new();
        let rock_no = 0;
        let jet_no = 0;
        let height = 0;

        if in_reader
            .read_line(&mut line)
            .expect("Failed to read input file")
            == 0
        {
            bail!("input file is empty!");
        }

        for c in line.trim().chars() {
            jet_pattern.push(match c {
                '<' => Jet::LEFT,
                '>' => Jet::RIGHT,
                _ => bail!("unknown jet direction {}", c),
            })
        }

        Ok(TetrisTower {
            jet_pattern,
            rock_pattern,
            tower,
            rock_no,
            jet_no,
            height,
        })
    }

    fn initial_rock_formation(&self, block: &TetrisBlock) -> Vec<(usize, u8)> {
        let x = self.tower.len() + 3;
        match block {
            TetrisBlock::HOR => vec![(x, 0b00111100u8)],
            TetrisBlock::PLUS => vec![
                (x, 0b00010000u8),
                (x + 1, 0b00111000u8),
                (x + 2, 0b00010000u8),
            ],
            TetrisBlock::MIRRORED => {
                vec![
                    (x, 0b00111000u8),
                    (x + 1, 0b00001000u8),
                    (x + 2, 0b00001000u8),
                ]
            }
            TetrisBlock::VERT => vec![
                (x, 0b00100000u8),
                (x + 1, 0b00100000u8),
                (x + 2, 0b00100000u8),
                (x + 3, 0b00100000u8),
            ],
            TetrisBlock::SQUARE => vec![(x, 0b00110000u8), (x + 1, 0b00110000u8)],
        }
    }

    fn move_by(&self, block: &Vec<(usize, u8)>, (x, y): (usize, i32)) -> Option<Vec<(usize, u8)>> {
        let mut new_block = Vec::new();
        for (b_x, row) in block {
            if *b_x == 0 && x == 1 {
                // hit bottom
                return None;
            }
            let next_x = b_x - x;
            let next_row = if y < 0 { row << y.abs() } else { row >> y };
            if (row & 0x80u8 != 0x00u8 && y < 0)
                || (next_row & 0x01u8 != 0x00u8 && y > 0)
                || (next_x < self.tower.len() && next_row & self.tower[next_x] != 0x00u8)
            {
                return None;
            }
            new_block.push((next_x, next_row));
        }
        Some(new_block)
    }

    fn update_tower(&mut self, mut block: Vec<(usize, u8)>) -> Result<()> {
        block.sort_by(|(x1, _), (x2, _)| x1.cmp(x2));
        let mut x_mod = 0;
        for (x, row) in block {
            let curr_x = x - x_mod;
            let do_split = if let Some(curr_row) = self.tower.get_mut(curr_x) {
                if *curr_row & row != 0x00u8 {
                    bail!("block can't be added here!");
                } else {
                    *curr_row = *curr_row | row;
                    *curr_row == 0b11111110u8 // split when row is full
                }
            } else {
                self.tower.push(row);
                false
            };
            if do_split {
                self.height += curr_x + 1;
                self.tower = self.tower.split_off(curr_x + 1);
                x_mod = x_mod + curr_x + 1;
            }
        }
        Ok(())
    }

    fn reset_state_to(&mut self, (tower, jet_no, rock_no): State, height: usize) {
        self.tower = tower;
        self.jet_no = jet_no;
        self.rock_no = rock_no;
        self.height = height;
    }

    fn reset_state(&mut self) {
        self.tower = Vec::new();
        self.rock_no = 0;
        self.jet_no = 0;
        self.height = 0;
    }

    fn get_identifier(&self) -> State {
        (self.tower.to_owned(), self.jet_no, self.rock_no)
    }
}

impl Iterator for TetrisTower {
    /// returns tuple of state identifier and current height of tower
    type Item = (State, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // drop next rock
        let mut block = self.initial_rock_formation(&self.rock_pattern[self.rock_no]);

        // iterate until rock comes to rest
        loop {
            let y_movement = match &self.jet_pattern[self.jet_no] {
                Jet::LEFT => -1,
                Jet::RIGHT => 1,
            };
            if let Some(new_block) = self.move_by(&block, (0, y_movement)) {
                block = new_block;
            }
            let moved_block = self.move_by(&block, (1, 0));
            self.jet_no = (self.jet_no + 1) % self.jet_pattern.len();
            if let Some(new_block) = moved_block {
                block = new_block;
            } else {
                break;
            }
        }

        self.rock_no = (self.rock_no + 1) % self.rock_pattern.len();

        // update tower
        self.update_tower(block).expect("failed to update tower");

        // return height
        Some((self.get_identifier(), self.height))
    }
}
