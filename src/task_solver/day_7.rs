use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use regex::Regex;
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
    u64,
};

pub fn solve(_task: u8, input: String) -> Result<()> {
    let root_dir =
        Directory::init_from_input(input).context("failed to instantiate file system")?;
    debug!("size of root dir: {}", root_dir.file_size);

    match _task {
        1 => info!(
            "sum of all total dir sizes of at most 100000: {}",
            root_dir.solve_1()
        ),
        2 => info!(
            "size of smallest possible dir that could free up enough space: {}",
            root_dir.solve_2(30000000 - (70000000 - root_dir.file_size))
        ),
        _ => return Err(anyhow!("task doesn't exist!")),
    }
    Ok(())
}

struct Directory {
    name: String,
    sub_dirs: Vec<Directory>,
    file_size: u64, // size of files contained at this dir level
}

impl Directory {
    fn init_from_input(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let mut in_reader = BufReader::new(in_file);
        let mut line = String::new();
        let mut dir_stack: VecDeque<Directory> = VecDeque::new(); // cd dir path

        let mut curr_dir = Directory {
            name: "stub".to_owned(),
            sub_dirs: vec![],
            file_size: 0u64,
        };

        // command regex
        let re_cd = Regex::new(r"^\$ cd (?P<dir_name>.*)").unwrap();
        let re_ls = Regex::new(r"^\$ ls").unwrap();
        let re_ls_dir = Regex::new(r"^dir (?P<dir_name>.*)").unwrap();
        let re_ls_file = Regex::new(r"^(?P<file_size>\d+) (?P<file_name>.*)").unwrap();

        loop {
            let bytes_read = in_reader
                .read_line(&mut line)
                .expect("Failed to read line in input file");
            if bytes_read == 0 || line == "\n" {
                // unwind until we reach root
                while curr_dir.name != "/" {
                    let mut prev_dir = dir_stack.pop_front().context("dir stack is empty!")?;
                    prev_dir.file_size += curr_dir.file_size;
                    prev_dir.sub_dirs.push(curr_dir);
                    curr_dir = prev_dir;
                }
                return Ok(curr_dir); // EOF
            }

            if re_cd.is_match(&line) {
                let cd_captures = re_cd
                    .captures(&line)
                    .context("cd regex failed to capture line")?;
                let dir_name = cd_captures
                    .name("dir_name")
                    .context("cd regex didn't contain expected named capture group")?
                    .as_str();
                if dir_name == ".." {
                    let mut prev_dir = dir_stack.pop_front().context("dir stack is empty!")?;
                    prev_dir.file_size += curr_dir.file_size;
                    debug!(
                        "final file size of {}: {}",
                        curr_dir.name, curr_dir.file_size
                    );
                    prev_dir.sub_dirs.push(curr_dir);
                    curr_dir = prev_dir;
                } else {
                    let cd_dir = Directory {
                        name: dir_name.to_owned(),
                        sub_dirs: vec![],
                        file_size: 0,
                    };
                    dir_stack.push_front(curr_dir);
                    curr_dir = cd_dir;
                }
                debug!("cmd cd - new curr dir: {}", dir_name);
            } else if re_ls.is_match(&line) {
                curr_dir.file_size = 0u64;
                debug!("cmd ls - curr dir: {}", curr_dir.name);
            } else if re_ls_dir.is_match(&line) {
                debug!("ls dir: {}", line);
            } else if re_ls_file.is_match(&line) {
                let ls_captures = re_ls_file
                    .captures(&line)
                    .context("ls_file regex failed to capture line")?;
                let file_size = ls_captures
                    .name("file_size")
                    .context("ls_file regex didn't contain expected named capture group")?
                    .as_str()
                    .parse::<u64>()
                    .context("failed to parse matched file size as u64")?;
                curr_dir.file_size += file_size;
                debug!("ls file: {}", line);
            } else {
                return Err(anyhow!("unknown terminal output: {}", line));
            }
            line.clear();
        }
    }

    fn solve_1(&self) -> u64 {
        let mut total_size = 0u64;
        for subdir in &self.sub_dirs {
            total_size += subdir.solve_1();
        }
        debug!("{} has size {}", self.name, self.file_size);
        if self.file_size <= 100000 {
            total_size += self.file_size;
        }
        total_size
    }

    fn solve_2(&self, required_space: u64) -> u64 {
        let mut curr_choice = self.file_size;
        for subdir in &self.sub_dirs {
            if subdir.file_size > required_space {
                let subdir_choice = subdir.solve_2(required_space);
                if curr_choice >= subdir_choice {
                    curr_choice = subdir_choice;
                }
            }
        }
        curr_choice
    }
}
