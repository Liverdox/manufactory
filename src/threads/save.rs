use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex, Condvar}, time::Duration};

use crate::{world::World, unsafe_mutex::UnsafeMutex, save_load::WorldRegions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SaveState {
    Unsaved,
    Saved,
    WorldExit,
}

pub fn spawn(
    world: Arc<UnsafeMutex<World>>,
    world_regions: Arc<UnsafeMutex<WorldRegions>>,
    save_condvar: Arc<(Mutex<SaveState>, Condvar)>
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let (lock, cvar) = &*save_condvar;
            let (mut save_state, _) = cvar.wait_timeout(lock.lock().unwrap(), Duration::new(60, 0)).unwrap();

            
            let mut world = unsafe {world.lock_unsafe()}.unwrap();
            let mut world_regions = unsafe {world_regions.lock_unsafe()}.unwrap();

            let mut chunks_awaiting_deletion = world.chunks.chunks_awaiting_deletion.lock().unwrap();
            chunks_awaiting_deletion.iter().for_each(|chunk| {
                world_regions.save_chunk(chunk);
            });
            chunks_awaiting_deletion.clear();
            drop(chunks_awaiting_deletion);

            world.chunks.chunks.iter_mut().for_each(|chunk| {
                let Some(chunk) = chunk else {return};
                if !chunk.unsaved {return};
                world_regions.save_chunk(chunk);
                chunk.unsaved = false;
            });
            world_regions.save_all_regions();


            if *save_state == SaveState::WorldExit {break};
            *save_state = SaveState::Saved;
        }
    })
}