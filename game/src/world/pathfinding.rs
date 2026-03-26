use crate::provider;
use crate::world::{Direction, Position};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

const MAX_PATH_LENGTH: usize = 64;

const DIRECTION_ORDER: [Direction; 8] = [
    Direction::West,
    Direction::East,
    Direction::South,
    Direction::North,
    Direction::SouthWest,
    Direction::SouthEast,
    Direction::NorthWest,
    Direction::NorthEast,
];

#[derive(Eq, PartialEq)]
struct Node {
    pos: (i32, i32),
    f: u32,
    g: u32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then_with(|| other.g.cmp(&self.g))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn find_path(start: Position, goal: Position) -> VecDeque<Position> {
    find_path_inner(start, goal, false)
}

pub fn find_path_adjacent(start: Position, goal: Position) -> VecDeque<Position> {
    find_path_inner(start, goal, true)
}

fn find_path_inner(start: Position, goal: Position, stop_adjacent: bool) -> VecDeque<Position> {
    if start == goal {
        return VecDeque::new();
    }

    if stop_adjacent && start.chebyshev_pos(goal) == 1 {
        return VecDeque::new();
    }

    let start_key = (start.x, start.y);
    let goal_key = (goal.x, goal.y);

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g_score: HashMap<(i32, i32), u32> = HashMap::new();

    g_score.insert(start_key, 0);
    came_from.insert(start_key, start_key);
    open.push(Node {
        pos: start_key,
        f: heuristic(start, goal),
        g: 0,
    });

    let mut closest_key = start_key;
    let mut closest_dist = heuristic(start, goal);

    while let Some(current) = open.pop() {
        let reached = if stop_adjacent {
            chebyshev(current.pos, goal_key) == 1
        } else {
            current.pos == goal_key
        };

        if reached {
            return reconstruct_path(&came_from, current.pos, start_key, start.plane);
        }

        let current_g = g_score[&current.pos];
        if current.g > current_g {
            continue;
        }
        if current_g >= MAX_PATH_LENGTH as u32 {
            continue;
        }

        let current_pos = Position::new(current.pos.0, current.pos.1, start.plane);

        for &dir in &DIRECTION_ORDER {
            if !provider::get_collision().can_move(current_pos, dir) {
                continue;
            }

            let neighbor = current_pos.step(dir);
            let neighbor_key = (neighbor.x, neighbor.y);
            let step_cost = if dir.is_diagonal() { 2 } else { 1 };
            let tentative_g = current_g + step_cost;

            if tentative_g >= g_score.get(&neighbor_key).copied().unwrap_or(u32::MAX) {
                continue;
            }

            g_score.insert(neighbor_key, tentative_g);
            came_from.insert(neighbor_key, current.pos);

            let h = heuristic(neighbor, goal);
            open.push(Node {
                pos: neighbor_key,
                f: tentative_g + h,
                g: tentative_g,
            });

            if h < closest_dist {
                closest_dist = h;
                closest_key = neighbor_key;
            }
        }
    }

    if closest_key != start_key {
        reconstruct_path(&came_from, closest_key, start_key, start.plane)
    } else {
        VecDeque::new()
    }
}

fn chebyshev(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn heuristic(a: Position, b: Position) -> u32 {
    let dx = (a.x - b.x).unsigned_abs();
    let dy = (a.y - b.y).unsigned_abs();
    let min = dx.min(dy);
    let max = dx.max(dy);
    min * 2 + (max - min)
}

fn reconstruct_path(
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    goal: (i32, i32),
    start: (i32, i32),
    plane: i32,
) -> VecDeque<Position> {
    let mut path = VecDeque::new();
    let mut current = goal;
    while current != start {
        path.push_front(Position::new(current.0, current.1, plane));
        current = came_from[&current];
    }
    path
}
