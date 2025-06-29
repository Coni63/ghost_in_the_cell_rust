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
const FRONTLINE_FACTORY: i32 = -1;
const FRONTLINE_DISTANCE: i32 = MAX_INT;
const CYBORGS_OWN: i32 = 0;
const CYBORGS_ENEMY: i32 = 0;
const ENEMY_CODE: i32 = -1;
const MY_CODE: i32 = 1;

type I32Matrix = [[i32; NUM_FACTORIES]; NUM_FACTORIES];
type UsizeMatrix = [[usize; NUM_FACTORIES]; NUM_FACTORIES];
type VecMatrix = [[Vec<usize>; NUM_FACTORIES]; NUM_FACTORIES];

struct Base {
    entity_id: usize,
    owner: i32,
    cyborgs: i32,
    production: i32,
    cooldown: i32,
    incomming: Vec<usize>, // List of incoming troops
    actions: Vec<Action>,  // Actions to be taken this turn
    blacklist: Vec<usize>, // Blacklist of factories that are not to be attacked
    troops_defensive: i32, // Local threshold Troops defending this base
    troops_offensive: i32, // Local threshold Troops attacking this base
}

impl Base {
    pub fn new(entity_id: usize) -> Self {
        Base {
            entity_id,
            owner: 0,
            cyborgs: 0,
            production: 0,
            cooldown: 0,
            incomming: Vec::new(),
            actions: Vec::new(),
            blacklist: Vec::new(),
            troops_defensive: 0,
            troops_offensive: 0,
        }
    }

    pub fn update(&mut self, owner: i32, cyborgs: i32, production: i32, cooldown: i32) {
        self.owner = owner;
        self.cyborgs = cyborgs;
        self.production = production;
        self.cooldown = cooldown;
    }

    pub fn tick(&mut self) {
        self.incomming.clear();
        self.actions.clear();
        self.blacklist.clear();
    }

    pub fn push_incomming_troop(&mut self, entity_id: usize) {
        self.incomming.push(entity_id);
    }

    pub fn closest_enemy(
        &self,
        dist_matrix: &I32Matrix,
        factories: &[Base],
    ) -> (Option<usize>, i32) {
        let mut nearest_factory: Option<usize> = None;
        let mut nearest_distance = MAX_INT;
        for (i, factory) in factories.iter().enumerate() {
            let distance = dist_matrix[self.entity_id][factory.entity_id];
            if distance < nearest_distance
                && i != self.entity_id
                && factories[i].owner != self.owner
            {
                nearest_distance = distance;
                nearest_factory = Some(i);
            }
        }
        (nearest_factory, nearest_distance)
    }
}

struct Bomb {
    entity_id: usize,
    owner: i32,
    source: usize,
    target: usize,
    cooldown: i32,
}

impl Bomb {}

struct Troop {
    entity_id: usize,
    owner: i32,
    source: usize,
    target: usize,
    cyborgs: i32,
    cooldown: i32,
}

impl Troop {}

enum Action {
    BOMB(i32, i32),
    TROOP(i32, i32, i32),
    WAIT,
    INC(i32),
}

/// Load all edges from the game input and return a pairwise matrix (symmetrical)
fn init_edge_matrix() -> I32Matrix {
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
fn floyd_warshall(bases_distances: &I32Matrix) -> (I32Matrix, UsizeMatrix, VecMatrix) {
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

fn find_frontline_factory(bases_distances: &I32Matrix, factories: &[Base]) -> Option<usize> {
    let mut frontline_factory: Option<usize> = None;

    // Find the closest enemy factory to any of our factories
    let mut frontline_distance = MAX_INT;
    for factory in factories.iter() {
        if factory.owner == ENEMY_CODE {
            let (_, nearest_distance) = factory.closest_enemy(bases_distances, factories);
            if nearest_distance < frontline_distance {
                frontline_distance = nearest_distance;
                frontline_factory = Some(factory.entity_id);
            }
        }
    }

    frontline_factory
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

    let mut factory_info: Vec<Base> = (0..factory_count).map(|i| Base::new(i)).collect();
    let mut factory_simul: Vec<Base> = (0..factory_count).map(|i| Base::new(i)).collect();

    loop {
        let mut count_simulations = 0;

        let mut troops: Vec<Troop> = vec![];
        let mut bombs: Vec<Bomb> = vec![];
        let mut actions: Vec<Action> = vec![];

        let mut cyborgs_own = 0;
        let mut cyborgs_enemy = 0;

        let mut my_factories: Vec<usize> = vec![];

        for i in 0..factory_count {
            factory_info[i].tick();
            factory_simul[i].tick();
        }

        // Reads game turn state
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let entity_count = parse_input!(input_line, i32); // the number of entities (e.g. factories and troops)
        for _ in 0..entity_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let entity_id = parse_input!(inputs[0], usize);
            let entity_type = inputs[1].trim().to_string();
            match entity_type.as_str() {
                "FACTORY" => {
                    let owner = parse_input!(inputs[2], i32);
                    let cyborgs = parse_input!(inputs[3], i32);
                    let production = parse_input!(inputs[4], i32);
                    let cooldown = parse_input!(inputs[5], i32);
                    factory_info[entity_id].update(owner, cyborgs, production, cooldown);
                    factory_simul[entity_id].update(owner, cyborgs, production, cooldown);

                    if owner == MY_CODE {
                        my_factories.push(entity_id);
                        cyborgs_own += cyborgs;
                    } else {
                        cyborgs_enemy += cyborgs;
                    }
                }
                "TROOP" => {
                    let owner = parse_input!(inputs[2], i32);
                    let source = parse_input!(inputs[3], usize);
                    let target = parse_input!(inputs[4], usize);
                    let cyborgs = parse_input!(inputs[5], i32);
                    let cooldown = parse_input!(inputs[6], i32);
                    troops.push(Troop {
                        entity_id,
                        owner,
                        source,
                        target,
                        cyborgs,
                        cooldown,
                    });
                    factory_info[target].push_incomming_troop(entity_id);
                    if owner == MY_CODE {
                        cyborgs_own += cyborgs;
                    } else {
                        cyborgs_enemy += cyborgs;
                    }
                }
                "BOMB" => bombs.push(Bomb {
                    entity_id,
                    owner: parse_input!(inputs[2], i32),
                    source: parse_input!(inputs[3], usize),
                    target: parse_input!(inputs[4], usize),
                    cooldown: parse_input!(inputs[5], i32),
                }),
                _ => panic!("Uncovered entity type"),
            }
        }

        let frontline_factory = find_frontline_factory(&bases_distances, &factory_info);
        eprintln!("Determined FRONTLINE factory: {frontline_factory:?}");

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

        println!("WAIT"); // Placeholder action, replace with actual logic
    }
}
