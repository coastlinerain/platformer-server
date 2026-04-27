use std::{collections::HashMap, iter::Map};

struct ServerWorld {
    players: HashMap<u64, PlayerData>,
    map: Map,
}

impl ServerWorld {
    fn update(&mut self) {}
}
