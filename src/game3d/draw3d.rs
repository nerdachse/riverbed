use std::collections::HashMap;
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use crate::agents::PlayerControlled;
use crate::blocs::{Blocs, ChunkPos, CHUNK_S1, Y_CHUNKS};
use crate::gen::{ColUnloadEvent, LoadArea};
use super::texture_array::{BlocTextureArray, TexState};
use super::{render3d::Meshable, texture_array::{TextureMap, TextureArrayPlugin}};
const CHUNK_S1_HF: f32 = (CHUNK_S1/2) as f32;
const CHUNK_AABB: Aabb = Aabb {
    center: Vec3A::new(CHUNK_S1_HF, CHUNK_S1_HF, CHUNK_S1_HF),
    half_extents: Vec3A::new(CHUNK_S1_HF, CHUNK_S1_HF, CHUNK_S1_HF)
};

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut chunk_ents: ResMut<ChunkEntities>,
) {
    for col_ev in ev_unload.read() {
        for i in 0..Y_CHUNKS {
            if let Some(ent) = chunk_ents.0.remove(&ChunkPos {
                x: col_ev.0.x,
                y: i as i32,
                z: col_ev.0.z,
                realm: col_ev.0.realm
            }) {
                commands.entity(ent).despawn();
            }
        }
    }
}

pub fn process_bloc_changes(
    mut commands: Commands,
    mesh_query: Query<&Handle<Mesh>>,
    load_area_query: Query<&LoadArea, With<PlayerControlled>>,
    mut blocs: ResMut<Blocs>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_ents: ResMut<ChunkEntities>,
    texture_map: Res<TextureMap>,
    bloc_tex_array: Res<BlocTextureArray>,
) {
    let Ok(load_area) = load_area_query.get_single() else {
        return;
    };

    if let Some(chunk) = blocs.changes.pop_front() {
        if !load_area.col_dists.contains_key(&chunk.into()) { return; }
        if let Some(ent) = chunk_ents.0.get(&chunk) {
            if let Ok(handle) = mesh_query.get_component::<Handle<Mesh>>(*ent) {
                if let Some(mesh) = meshes.get_mut(handle) {
                    blocs.update_mesh(chunk, mesh, &texture_map);
                }
            } else {
                // the entity is not instanciated yet, we put it back
                blocs.changes.push_back(chunk);
            }
        } else {
            let ent = commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(blocs.create_mesh(chunk, &texture_map)),
                material: bloc_tex_array.0.clone(),
                transform: Transform::from_translation(
                    Vec3::new(chunk.x as f32, chunk.y as f32, chunk.z as f32) * CHUNK_S1 as f32 - Vec3::new(1., 1., 1.),
                ),
                ..Default::default()
            }).insert(CHUNK_AABB).id();
            chunk_ents.0.insert(chunk, ent);
        }
    }
}


#[derive(Resource)]
pub struct ChunkEntities(pub HashMap::<ChunkPos, Entity>);

impl ChunkEntities {
    pub fn new() -> Self {
        ChunkEntities(HashMap::new())
    }
}

pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TextureArrayPlugin)
            .insert_resource(ChunkEntities::new())
            .add_systems(Update, on_col_unload)
            // TODO: need to thread this so it can run as fast as possible but in the meantime running it twice is decent
            .add_systems(Update, process_bloc_changes.run_if(in_state(TexState::Finished)))
            .add_systems(Update, process_bloc_changes.run_if(in_state(TexState::Finished)))
            ;
    }
}
