use crate::gen::{earth_gen::Earth, debug_gen::DebugGen};
use crate::blocs::{Realm, ColPos, Blocs};
use bevy::prelude::Resource;
use std::collections::HashMap;

pub trait TerrainGen: Send + Sync {
    fn set_config(&mut self, config: HashMap<String, f32>);

    fn set_seed(&mut self, seed: u32);

    fn gen(&self, world: &mut Blocs, col: ColPos);
}

#[derive(Resource)]
pub struct Generators {
    data: HashMap<Realm, Box<dyn TerrainGen>>,
}

impl Generators {
    pub fn new(seed: u32) -> Self {
        let mut gens: HashMap<Realm, Box<dyn TerrainGen>> = HashMap::new();
        gens.insert(Realm::Overworld, Box::new(Earth::new(seed, HashMap::new())));
        Generators { data: gens }
    }

    pub fn gen(&self, world: &mut Blocs, pos: ColPos) {
        self.data.get(&Realm::Overworld).unwrap().gen(world, pos)
    }
}