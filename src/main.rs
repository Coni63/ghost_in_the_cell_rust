use std::time::{Duration, Instant};
use std::{array, io};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

// Configuration
const MAX_INT: i32 = i16::MAX as i32;
const SIMULATE_ENEMY: bool = true;

// Target Scoring Constants
const PRODUCTION_MULTIPLIER: i32 = 10;
const BOMB_SCORE_THRESHOLD: i32 = 1;
const BOMB_TROOP_THRESHOLD: i32 = 25;

// Movement Constants
const TROOP_OFFENSIVE: f32 = 1.00; // Sends this % of troops against superior enemies
const TROOP_DEFENSIVE: f32 = 1.00; // Sends this % of troops to reinforce friendly targets
const TROOP_OFFENSIVE_MULTIPLIER: f32 = 1.17;
const TROOP_EXCESS_NEUTRAL: f32 = 1.0;
const TROOP_EXCESS_ENEMY: f32 = 1.0;
const ENEMY_OFFENSIVE: f32 = 1.53; // How offensive is the enemy
const ENEMY_DEFENSIVE: f32 = 1.00; // How defensive is the enemy
const ENEMY_EXCESS_NEUTRAL: f32 = 1.0;
const ENEMY_EXCESS_ENEMY: f32 = 1.0;

// Game Variables
const NUM_FACTORIES: usize = 15;
const FACTORY_UPGRADE_COST: i32 = 10;
const BOMB_PRODUCTION_COOLDOWN: i32 = 5;
const MAX_LINK_DISTANCE: i32 = 7;
const INITIAL_FACTORY: i32 = -1;
const INITIAL_FACTORY_ENEMY: i32 = -1;
const FRONTLINE_FACTORY: i32 = -1;
const FRONTLINE_DISTANCE: i32 = MAX_INT;
const CYBORGS_OWN: i32 = 0;
const CYBORGS_ENEMY: i32 = 0;

type I32Matrix = [[i32; NUM_FACTORIES]; NUM_FACTORIES];
type UsizeMatrix = [[usize; NUM_FACTORIES]; NUM_FACTORIES];
type VecMatrix = [[Vec<usize>; NUM_FACTORIES]; NUM_FACTORIES];

/// Load all edges from the game input and return a pairwise matrix (symmetrical)
pub fn init_edge_matrix() -> I32Matrix {
    let mut bases_distances = [[0; NUM_FACTORIES]; NUM_FACTORIES];

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let link_count = parse_input!(input_line, i32); // the number of links between factories
    for _ in 0..link_count as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let factory_1 = parse_input!(inputs[0], usize);
        let factory_2 = parse_input!(inputs[1], usize);
        let distance = parse_input!(inputs[2], i32);
        // eprintln!("{} => {} = {}", factory_1, factory_2, distance);

        bases_distances[factory_1][factory_2] = distance;
        bases_distances[factory_2][factory_1] = distance;
    }
    bases_distances
}

/// Runs Floyd-Warshall with path and next matrix generation
pub fn floyd_warshall(bases_distances: &I32Matrix) -> (I32Matrix, UsizeMatrix, VecMatrix) {
    let mut dist = [[MAX_INT; NUM_FACTORIES]; NUM_FACTORIES];
    let mut next = [[MAX_INT as usize; NUM_FACTORIES]; NUM_FACTORIES];
    let mut path = array::from_fn(|_| array::from_fn(|_| Vec::<usize>::new()));

    // Initialize with base distances
    for i in 0..NUM_FACTORIES {
        for j in 0..NUM_FACTORIES {
            if i == j {
                dist[i][j] = MAX_INT;
            } else if bases_distances[i][j] < MAX_INT {
                dist[i][j] = bases_distances[i][j];
                next[i][j] = j;
                path[i][j].push(j);
            }
        }
    }

    // Floyd-Warshall algorithm with path reconstruction
    for k in 0..NUM_FACTORIES {
        for i in 0..NUM_FACTORIES {
            for j in 0..NUM_FACTORIES {
                if i == j || j == k || dist[i][k] == MAX_INT || dist[k][j] == MAX_INT {
                    continue;
                }

                let through_k = dist[i][k].saturating_add(dist[k][j]);
                if through_k < dist[i][j] {
                    dist[i][j] = through_k;
                    let intermediate_path = path[k][j].clone();
                    path[i][j].clear();
                    path[i][j].push(k);
                    path[i][j].extend_from_slice(&intermediate_path);
                    next[i][j] = next[i][k];
                }
            }
        }
    }

    (dist, next, path)
}

fn main() {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let factory_count = parse_input!(input_line, usize); // the number of factories

    // bases_distances: Matrix to store edges distances befaore optimization with floyd
    let bases_distances = init_edge_matrix();

    // floyd_war_matrix: Matrix to store shortest distances
    // floyd_war_path: Matrix to store the path between 2 nodes
    // floyd_war_next: Matrix to store the next node between 2 nodes
    let (floyd_war_matrix, floyd_war_next, floyd_war_path) = floyd_warshall(&bases_distances);
    // eprintln!("{:?}", bases_distances);
    // eprintln!("{:?}", floyd_war_matrix);
    // eprintln!("{:?}", floyd_war_next);
    // eprintln!("{:?}", floyd_war_path);

    // game loop
    let mut turn = 0;

    // let mut factory_info: [Base; 15] = (0..15).map(|i| Base::new(i));
    // let mut factory_simul: [Base; 15] = (0..15).map(|i| Base::new(i));

    // loop {
    //     let mut troops: Vec<Troop> = vec![];
    //     let mut bombs: Vec<Bomb> = vec![];
    //     let mut factories: Vec<Base> = vec![];

    //     let mut input_line = String::new();
    //     io::stdin().read_line(&mut input_line).unwrap();
    //     let entity_count = parse_input!(input_line, i32); // the number of entities (e.g. factories and troops)
    //     for _ in 0..entity_count as usize {
    //         let mut input_line = String::new();
    //         io::stdin().read_line(&mut input_line).unwrap();
    //         let inputs = input_line.split(" ").collect::<Vec<_>>();
    //         let entity_id = parse_input!(inputs[0], i32);
    //         let entity_type = inputs[1].trim().to_string();
    //         match entity_type.as_str() {
    //             "FACTORY" => factories.push(Base {
    //                 entity_id,
    //                 owner: parse_input!(inputs[2], i32),
    //                 cyborgs: parse_input!(inputs[3], i32),
    //                 production: parse_input!(inputs[4], i32),
    //                 timeout: parse_input!(inputs[5], i32),
    //             }),
    //             "TROOP" => troops.push(Troop {
    //                 entity_id,
    //                 owner: parse_input!(inputs[2], i32),
    //                 source: parse_input!(inputs[3], i32),
    //                 target: parse_input!(inputs[4], i32),
    //                 cyborgs: parse_input!(inputs[5], i32),
    //                 timeout: parse_input!(inputs[6], i32),
    //             }),
    //             "BOMB" => bombs.push(Bomb {
    //                 entity_id,
    //                 owner: parse_input!(inputs[2], i32),
    //                 source: parse_input!(inputs[3], i32),
    //                 target: parse_input!(inputs[4], i32),
    //                 timeout: parse_input!(inputs[5], i32),
    //             }),
    //             _ => panic!("Uncovered entity type"),
    //         }
    //     }

    //     let actions = solve(
    //         turn,
    //         &bases_distances,
    //         &distances,
    //         &next,
    //         &centrality,
    //         &factories,
    //         &troops,
    //     );

    //     // Write an action using println!("message...");
    //     // To debug: eprintln!("Debug message...");

    //     // Any valid action, such as "WAIT" or "MOVE source destination cyborgs"
    //     println!("{}", stringify_actions(&actions));
    //     turn += 1;
    // }
}
