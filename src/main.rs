use std::io;
use std::time::{Duration, Instant};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

enum Action {
    BOMB(i32, i32),
    TROOP(i32, i32, i32),
    WAIT,
    INC(i32),
}

struct Base {
    entity_id: i32,
    owner: i32,
    cyborgs: i32,
    production: i32,
    timeout: i32,
}

impl Base {}

struct Bomb {
    entity_id: i32,
    owner: i32,
    source: i32,
    target: i32,
    timeout: i32,
}

impl Bomb {}

struct Troop {
    entity_id: i32,
    owner: i32,
    source: i32,
    target: i32,
    cyborgs: i32,
    timeout: i32,
}

impl Troop {}

fn get_optimal_paths(
    dist: &[[i32; 15]; 15],
    num_bases: usize,
) -> ([[i32; 15]; 15], [[i32; 15]; 15]) {
    // Initialize distances matrix with input distances
    let mut distances = [[99999; 15]; 15];
    let mut next = [[99999; 15]; 15];

    // Copy the relevant part of the input matrix and ensure symmetry
    for i in 0..num_bases {
        distances[i][i] = 0;
        for j in i + 1..num_bases {
            // Take the minimum of both directions to ensure symmetry
            let min_dist = std::cmp::min(dist[i][j], dist[j][i]);
            distances[i][j] = min_dist;
            distances[j][i] = min_dist;

            if min_dist != 99999 {
                next[i][j] = j as i32;
                next[j][i] = i as i32;
            }
        }
    }

    // Floyd-Warshall algorithm
    for k in 0..num_bases {
        for i in 0..num_bases {
            for j in i + 1..num_bases {
                // Only process upper triangle
                let new_dist = distances[i][k]
                    .saturating_add(distances[k][j])
                    .saturating_add(1);
                if new_dist < distances[i][j] {
                    // Update both directions
                    distances[i][j] = new_dist;
                    distances[j][i] = new_dist;

                    // Update next matrix for both directions
                    next[i][j] = next[i][k];
                    next[j][i] = next[j][k];
                }
            }
        }
    }

    (distances, next)
}

// Helper function to reconstruct the path
fn get_path(start: usize, end: usize, next: &[[i32; 15]; 15]) -> Vec<usize> {
    if next[start][end] == 99999 {
        return vec![];
    }

    let mut path = vec![start];
    let mut current = start;

    while current != end {
        current = next[current][end] as usize;
        path.push(current);
    }

    path
}

fn calculate_node_centrality(num_bases: usize, next: &[[i32; 15]; 15]) -> [i32; 15] {
    let mut path_counts = [0; 15];

    // Count paths passing through each node
    for start in 0..num_bases {
        for end in start + 1..num_bases {
            let path = get_path(start, end, next);
            // Count intermediate nodes (exclude start and end)
            for &node in path.iter().skip(1).take(path.len().saturating_sub(2)) {
                path_counts[node] += 2;
            }
        }
    }

    path_counts
}

fn calculate_all_edges(next: &[[i32; 15]; 15], num_bases: usize) -> Vec<(usize, usize)> {
    let mut ans = vec![];

    for start in 0..num_bases {
        let mut visited = [false; 15];
        for end in start + 1..num_bases {
            let next_node = next[start][end] as usize;
            let t = (start, next_node);
            if !visited[next_node] && !ans.contains(&t) {
                ans.push(t);
                ans.push((next_node, start));
                visited[next_node] = true;
            }
        }
    }

    ans
}

fn solve(
    turn: i32,
    bases_distances: &[[i32; 15]; 15],
    distances: &[[i32; 15]; 15],
    next: &[[i32; 15]; 15],
    centrality: &[i32; 15],
    factories: &[Base],
    troops: &[Troop],
) -> Vec<Action> {
    let mut actions = vec![Action::WAIT];
    actions.extend(get_bomb_actions(turn, bases_distances, factories));
    actions.extend(get_best_moved(
        distances, next, centrality, factories, troops,
    ));
    actions
}

fn get_bomb_actions(
    turn: i32,
    bases_distances: &[[i32; 15]; 15],
    factories: &[Base],
) -> Vec<Action> {
    if turn == 0 || turn == 5 {
        // Filter opponent bases (owner == -1), sort by production (descending), and retain only the top producers
        let mut opponent_bases: Vec<&Base> =
            factories.iter().filter(|base| base.owner == -1).collect();

        // Sort the filtered bases by production in descending order
        opponent_bases.sort_by(|a, b| b.production.cmp(&a.production));

        // Retain only those bases with the highest production
        let max_production = opponent_bases.first().map_or(0, |base| base.production);
        let top_opponent_bases: Vec<&Base> = opponent_bases
            .into_iter()
            .take_while(|&base| base.production == max_production)
            .collect();

        let my_bases: Vec<&Base> = factories.iter().filter(|base| base.owner == 1).collect();

        let mut closest = 999;
        let mut best = (0, 0);
        for my_base in my_bases {
            for opp_bases in top_opponent_bases.iter() {
                let d = bases_distances[my_base.entity_id as usize][opp_bases.entity_id as usize];
                if d < closest {
                    closest = d;
                    best = (my_base.entity_id, opp_bases.entity_id)
                }
            }
        }

        return vec![Action::BOMB(best.0, best.1)];
    }

    vec![]
}

fn get_best_moved(
    distances: &[[i32; 15]; 15],
    next: &[[i32; 15]; 15],
    centrality: &[i32; 15],
    factories: &[Base],
    troops: &[Troop],
) -> Vec<Action> {
    let duration = Duration::from_millis(48);
    let timer = Instant::now();

    while timer.elapsed() < duration {}

    eprintln!("{:?}", timer.elapsed());

    vec![]
}

fn stringify_actions(actions: &[Action]) -> String {
    let mut ans: Vec<String> = vec![];

    for action in actions {
        let s = match action {
            Action::BOMB(from, to) => format!("BOMB {} {}", from, to),
            Action::TROOP(from, to, count) => format!("TROOP {} {} {}", from, to, count),
            Action::WAIT => String::from("WAIT"),
            Action::INC(base) => format!("INC {}", base),
        };
        ans.push(s);
    }

    ans.join(";")
}

fn main() {
    let mut turn = 0;
    let mut bases_distances: [[i32; 15]; 15] = [[99999; 15]; 15];
    for i in 0..15 {
        bases_distances[i][i] = 0;
    }

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let factory_count = parse_input!(input_line, usize); // the number of factories
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let link_count = parse_input!(input_line, i32); // the number of links between factories
    for i in 0..link_count as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let factory_1 = parse_input!(inputs[0], i32);
        let factory_2 = parse_input!(inputs[1], i32);
        let distance = parse_input!(inputs[2], i32);
        // eprintln!("{} => {} = {}", factory_1, factory_2, distance);

        bases_distances[factory_1 as usize][factory_2 as usize] = distance;
        bases_distances[factory_2 as usize][factory_1 as usize] = distance;
    }

    // distances: pairwise distance matrix with optimized path
    let (distances, next) = get_optimal_paths(&bases_distances, factory_count);
    let centrality = calculate_node_centrality(factory_count, &next);
    // let all_edges = calculate_all_edges(&next, factory_count);

    // eprintln!("{:?}", bases_distances);
    // eprintln!("{:?}", distances);
    // eprintln!("{:?}", next);
    // for i in 0..factory_count {
    //     for j in i+1..factory_count {
    //         eprintln!("{:?} vs {:?}", get_path(i, j, &next), get_path(j, i, &next));
    //     }
    // }
    // eprintln!("{:?}", centrality);
    // eprintln!("{:?}", all_edges);
    // eprintln!("{} vs {}", link_count, all_edges.len());

    // game loop
    loop {
        let mut troops: Vec<Troop> = vec![];
        let mut bombs: Vec<Bomb> = vec![];
        let mut factories: Vec<Base> = vec![];

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let entity_count = parse_input!(input_line, i32); // the number of entities (e.g. factories and troops)
        for _ in 0..entity_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let entity_id = parse_input!(inputs[0], i32);
            let entity_type = inputs[1].trim().to_string();
            match entity_type.as_str() {
                "FACTORY" => factories.push(Base {
                    entity_id,
                    owner: parse_input!(inputs[2], i32),
                    cyborgs: parse_input!(inputs[3], i32),
                    production: parse_input!(inputs[4], i32),
                    timeout: parse_input!(inputs[5], i32),
                }),
                "TROOP" => troops.push(Troop {
                    entity_id,
                    owner: parse_input!(inputs[2], i32),
                    source: parse_input!(inputs[3], i32),
                    target: parse_input!(inputs[4], i32),
                    cyborgs: parse_input!(inputs[5], i32),
                    timeout: parse_input!(inputs[6], i32),
                }),
                "BOMB" => bombs.push(Bomb {
                    entity_id,
                    owner: parse_input!(inputs[2], i32),
                    source: parse_input!(inputs[3], i32),
                    target: parse_input!(inputs[4], i32),
                    timeout: parse_input!(inputs[5], i32),
                }),
                _ => panic!("Uncovered entity type"),
            }
        }

        let actions = solve(
            turn,
            &bases_distances,
            &distances,
            &next,
            &centrality,
            &factories,
            &troops,
        );

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");

        // Any valid action, such as "WAIT" or "MOVE source destination cyborgs"
        println!("{}", stringify_actions(&actions));
        turn += 1;
    }
}
