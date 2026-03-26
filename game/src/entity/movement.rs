use crate::entity::Entity;
use crate::provider;
use crate::world::{Direction, Position};

impl Entity {
    pub fn step(&mut self) -> Option<Direction> {
        let next = self.walk_queue.pop_front()?;
        let dir = self.position.direction_to(next)?;

        if !provider::get_collision().can_move(self.position, dir) {
            self.walk_queue.clear();
            return None;
        }

        self.position = next;
        self.face_direction = dir;
        Some(dir)
    }

    pub fn peek_run_step(&self) -> Option<Direction> {
        let &run_pos = self.walk_queue.front()?;
        let run_dir = self.position.direction_to(run_pos)?;

        provider::get_collision()
            .can_move(self.position, run_dir)
            .then_some(run_dir)
    }

    pub fn commit_run_step(&mut self, run_dir: Direction) {
        let run_pos = self.walk_queue.pop_front().expect("no run step to commit");
        self.position = run_pos;
        self.face_direction = run_dir;
    }

    pub fn walk_to(&mut self, dest: Position, stop_adjacent: bool) {
        self.walk_queue = if stop_adjacent {
            crate::world::find_path_adjacent(self.position, dest)
        } else {
            crate::world::find_path(self.position, dest)
        };
    }

    pub fn stop(&mut self) {
        self.walk_queue.clear();
    }

    pub fn has_steps(&self) -> bool {
        !self.walk_queue.is_empty()
    }
}
