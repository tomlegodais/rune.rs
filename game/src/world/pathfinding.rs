use std::collections::{HashMap, VecDeque};

use crate::{
    provider,
    world::{Direction, Position, collision::CollisionMap},
};

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

const WALL_W: u32 = 0x80;
const WALL_E: u32 = 0x8;
const WALL_S: u32 = 0x2;
const WALL_N: u32 = 0x20;

pub fn find_path(start: Position, goal: Position) -> VecDeque<Position> {
    if start == goal {
        return VecDeque::new();
    }

    let goal_key = (goal.x, goal.y);
    let (came_from, dist, start_key) = bfs_fill(start);

    if let Some(&_) = dist.get(&goal_key) {
        return reconstruct_path(&came_from, goal_key, start_key, start.plane);
    }

    walk_closest(&came_from, &dist, start_key, goal_key, start.plane)
}

pub fn find_path_adjacent_rect(
    start: Position,
    goal: Position,
    width: i32,
    height: i32,
    access_block_flag: u8,
) -> VecDeque<Position> {
    let collision = provider::get_collision();
    if can_interact_rect(collision, start, goal, width, height, access_block_flag) {
        return VecDeque::new();
    }

    let (came_from, dist, start_key) = bfs_fill(start);

    let mut best: Option<((i32, i32), u32)> = None;
    let mut try_candidate = |key: (i32, i32), boundary: Position, wall_flag: u32| {
        if let Some(&d) = dist.get(&key)
            && collision.flag_at(boundary) & wall_flag == 0
            && best.is_none_or(|(_, bd)| d < bd)
        {
            best = Some((key, d));
        }
    };

    if access_block_flag & 0x8 == 0 {
        for ty in goal.y..goal.y + height {
            try_candidate((goal.x - 1, ty), Position::new(goal.x, ty, start.plane), WALL_W);
        }
    }

    if access_block_flag & 0x2 == 0 {
        for ty in goal.y..goal.y + height {
            try_candidate(
                (goal.x + width, ty),
                Position::new(goal.x + width - 1, ty, start.plane),
                WALL_E,
            );
        }
    }

    if access_block_flag & 0x1 == 0 {
        for tx in goal.x..goal.x + width {
            try_candidate((tx, goal.y - 1), Position::new(tx, goal.y, start.plane), WALL_S);
        }
    }

    if access_block_flag & 0x4 == 0 {
        for tx in goal.x..goal.x + width {
            try_candidate(
                (tx, goal.y + height),
                Position::new(tx, goal.y + height - 1, start.plane),
                WALL_N,
            );
        }
    }

    match best {
        Some((key, _)) => reconstruct_path(&came_from, key, start_key, start.plane),
        None => walk_closest(&came_from, &dist, start_key, (goal.x, goal.y), start.plane),
    }
}

pub fn can_interact_rect(
    collision: &CollisionMap,
    pos: Position,
    target: Position,
    width: i32,
    height: i32,
    access_block_flag: u8,
) -> bool {
    let (px, py) = (pos.x, pos.y);
    let (tx, ty) = (target.x, target.y);
    let (tx2, ty2) = (tx + width, ty + height);

    if px == tx - 1 && py >= ty && py < ty2 && access_block_flag & 0x8 == 0 {
        return collision.flag_at(Position::new(tx, py, pos.plane)) & WALL_W == 0;
    }
    if px == tx2 && py >= ty && py < ty2 && access_block_flag & 0x2 == 0 {
        return collision.flag_at(Position::new(tx2 - 1, py, pos.plane)) & WALL_E == 0;
    }
    if py == ty - 1 && px >= tx && px < tx2 && access_block_flag & 0x1 == 0 {
        return collision.flag_at(Position::new(px, ty, pos.plane)) & WALL_S == 0;
    }
    if py == ty2 && px >= tx && px < tx2 && access_block_flag & 0x4 == 0 {
        return collision.flag_at(Position::new(px, ty2 - 1, pos.plane)) & WALL_N == 0;
    }

    false
}

type Tile = (i32, i32);
type BfsResult = (HashMap<Tile, Tile>, HashMap<Tile, u32>, Tile);

fn bfs_fill(start: Position) -> BfsResult {
    let start_key = (start.x, start.y);
    let collision = provider::get_collision();

    let mut queue = VecDeque::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut dist: HashMap<(i32, i32), u32> = HashMap::new();

    dist.insert(start_key, 0);
    came_from.insert(start_key, start_key);
    queue.push_back(start_key);

    while let Some(current_key) = queue.pop_front() {
        let current_d = dist[&current_key];
        if current_d >= MAX_PATH_LENGTH as u32 {
            continue;
        }

        let current_pos = Position::new(current_key.0, current_key.1, start.plane);

        for &dir in &DIRECTION_ORDER {
            if !collision.can_move(current_pos, dir) {
                continue;
            }

            let neighbor = current_pos.step(dir);
            let neighbor_key = (neighbor.x, neighbor.y);

            if dist.contains_key(&neighbor_key) {
                continue;
            }

            dist.insert(neighbor_key, current_d + 1);
            came_from.insert(neighbor_key, current_key);
            queue.push_back(neighbor_key);
        }
    }

    (came_from, dist, start_key)
}

fn walk_closest(
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    dist: &HashMap<(i32, i32), u32>,
    start_key: (i32, i32),
    goal_key: (i32, i32),
    plane: i32,
) -> VecDeque<Position> {
    let mut closest_key = start_key;
    let mut closest_h = chebyshev(start_key, goal_key);

    for &key in dist.keys() {
        let h = chebyshev(key, goal_key);
        if h < closest_h {
            closest_h = h;
            closest_key = key;
        }
    }

    if closest_key != start_key { reconstruct_path(came_from, closest_key, start_key, plane) } else { VecDeque::new() }
}

fn chebyshev(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
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
