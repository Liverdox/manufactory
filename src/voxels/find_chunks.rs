use std::time::Instant;

use itertools::iproduct;

use crate::{world::chunk_coords::ChunkCoords, rev_qumark};

use super::{chunks::{Chunks, WORLD_HEIGHT}, chunk::{Chunk, CHUNK_SIZE}};

const SIDE_COORDS_OFFSET: [(i32, i32, i32); 4] = [
    (1,0,0), (-1,0,0),
    (0,0,1), (0,0,-1),
];

impl Chunks {
    pub fn find_unloaded(&mut self) -> Option<(i32, i32)> {
        let callback = |cx: i32, cz: i32| {
            let index = ChunkCoords(cx, 0, cz).index_without_offset(self.width, self.depth);
            unsafe {self.chunks.get_unchecked(index)}.is_none().then_some((cx + self.ox, cz + self.oz))
        };
 
        Self::clockwise_square_spiral(self.width as usize, callback)
    }

    pub fn find_unrendered(&mut self) -> Option<&mut Chunk> {
        let callback = |cx: i32, cz: i32| {
            for cy in 0..WORLD_HEIGHT as i32 {
                let index = ChunkCoords(cx+1, cy, cz+1).index_without_offset(self.width, self.depth);
                if unsafe {self.chunks.get_unchecked(index)}.as_ref()
                    .map_or(true, |c| !c.modified()) {continue};
    
                let mut around_count = 0;
                for (ox, oy, oz) in SIDE_COORDS_OFFSET.into_iter() {
                    let index = ChunkCoords(cx + ox + 1, cy + oy, cz + oz + 1)
                        .index_without_offset(self.width, self.depth);
                    if self.chunks[index].is_some() {around_count += 1}
                }
                if around_count == 4 {return Some(index)}
            }
            None
        };

        Self::clockwise_square_spiral(self.width as usize - 2, callback)
            .and_then(|index| self.chunks[index].as_mut().map(|c| c.as_mut()))
    }

    pub fn clockwise_square_spiral<T>(n: usize, callback: impl Fn(i32, i32) -> Option<T>) -> Option<T> {
        let mut x = 0;
        let mut y = 0;
        let mut dx = 0;
        let mut dy = -1;
        let o = n as i32 % 2 ^ 1;
        let half = n as i32/2;
        for _ in 0..n.pow(2) {
            if x >= -half && x <= half && y >= -half && y <= half {
                let result = callback(x+half-o, y+half-o);
                if result.is_some() {return result};
            }
            if (x == y) || (x == -y && x < 0) || (x == 1-y && x > 0) {
                (dx, dy) = (-dy, dx);
            }
            x += dx;
            y += dy;
        }
        None
    }
}