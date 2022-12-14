use anyhow::{anyhow, Context, Ok, Result};
use log::{debug, info};
use petgraph::{algo::dijkstra, graph::NodeIndex, Directed, Graph};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub fn solve(task: u8, input: String) -> Result<()> {
    let heightmap = parse_heightmap(input).context("Failed to instantiate heightmap")?;

    let (path_graph, s, e) =
        grid_to_graph(heightmap).context("failed to convert the parsed heightmap to graph")?;

    debug!("graph looks like: {:#?}", path_graph);

    let sp_map = dijkstra(&path_graph, e, None, |_| 1);

    debug!("sp map: {:#?}", sp_map);

    let shortest_dist = sp_map
        .iter()
        .filter(|(node_id, _)| match task {
            1 => **node_id == s,
            2 => path_graph[**node_id] == 'a',
            _ => panic!("task doesn't exist!"),
        })
        .map(|(_, len)| len)
        .min()
        .ok_or(anyhow!(
            "shortest path map didn't contain any paths to node E: {:?}",
            sp_map
        ))?;

    info!("shortest path to E has length {}", shortest_dist);

    Ok(())
}

type HeightMap = Vec<Vec<char>>;
type Paths = Graph<char, (), Directed>;

fn parse_heightmap(input: String) -> Result<HeightMap> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut heightmap = HeightMap::new();

    loop {
        let bytes_read = in_reader
            .read_line(&mut line)
            .expect("Failed to read line in input file");
        if bytes_read == 0 || line == "\n" {
            break; // EOF
        }

        heightmap.push(line.trim().chars().collect());

        line.clear();
    }

    Ok(heightmap)
}

fn grid_to_graph(heightmap: HeightMap) -> Result<(Paths, NodeIndex, NodeIndex)> {
    let grid_h = heightmap.len();
    let grid_w = heightmap[0].len();

    let mut graph = Paths::new();
    let mut index_to_node_id = Vec::new();
    let mut node_id_s = 0.into();
    let mut node_id_e = 0.into();

    // initialize graph
    for row in heightmap.iter() {
        let mut node_map = Vec::new();
        for node in row.iter() {
            debug!("adding node '{}' to graph", node);
            match node {
                'S' => {
                    debug!("found node S!");
                    let node_id = graph.add_node('a');
                    node_map.push(node_id);
                    node_id_s = node_id;
                }
                'E' => {
                    let node_id = graph.add_node('z');
                    node_map.push(node_id);
                    node_id_e = node_id;
                }
                _ => node_map.push(graph.add_node(*node)),
            }
        }
        index_to_node_id.push(node_map);
    }

    // add graph edges
    for i in 0..grid_h {
        for j in 0..grid_w {
            if j < grid_w - 1 {
                add_edge_if_possible(&mut graph, &index_to_node_id, (i, j), (i, j + 1))
                    .context("failed when adding edges")?;
            }
            if i < grid_h - 1 {
                add_edge_if_possible(&mut graph, &index_to_node_id, (i, j), (i + 1, j))
                    .context("failed when adding edges")?;
            }
        }
    }

    Ok((graph, node_id_s, node_id_e))
}

fn char_to_int(c: char) -> i32 {
    c as i32 - 'a' as i32
}

fn add_edge_if_possible(
    graph: &mut Paths,
    index_to_node_id: &Vec<Vec<NodeIndex>>,
    (c_i, c_j): (usize, usize),
    (d_i, d_j): (usize, usize),
) -> Result<()> {
    let c_id = *index_to_node_id
        .get(c_i)
        .ok_or(anyhow!("c row index out of bounds"))?
        .get(c_j)
        .ok_or(anyhow!("c col index out of bounds"))?;
    let d_id = *index_to_node_id
        .get(d_i)
        .ok_or(anyhow!("d row index out of bounds"))?
        .get(d_j)
        .ok_or(anyhow!("d dol index out of bounds"))?;

    let c_elevation = graph
        .node_weight(c_id)
        .ok_or(anyhow!("failed to access node elevation"))?
        .to_owned();
    let d_elevation = graph
        .node_weight(d_id)
        .ok_or(anyhow!("failed to access node elevation"))?
        .to_owned();

    let c_elevation_val = char_to_int(c_elevation);
    let d_elevation_val = char_to_int(d_elevation);

    if (d_elevation_val - c_elevation_val).abs() <= 1 {
        // both directions
        graph.add_edge(c_id, d_id, ());
        graph.add_edge(d_id, c_id, ());
        debug!(
            "Added edge: ({:#?},{}) <--> ({:#?},{})",
            c_id, c_elevation, d_id, d_elevation
        );
    } else if d_elevation_val < c_elevation_val {
        // dest -> curr
        graph.add_edge(d_id, c_id, ());
        debug!(
            "Added edge: ({:#?},{}) <--- ({:#?},{})",
            c_id, c_elevation, d_id, d_elevation
        );
    } else {
        // current -> dest
        graph.add_edge(c_id, d_id, ());
        debug!(
            "Added edge: ({:#?},{}) ---> ({:#?},{})",
            c_id, c_elevation, d_id, d_elevation
        );
    }

    Ok(())
}
