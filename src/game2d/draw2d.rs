use crate::blocs::{CHUNK_S1, Bloc, Blocs, ColPos};
use crate::gen::{ColUnloadEvent, LoadedCols};
use crate::agents::Dir;
use anyhow::Result;
use bevy::prelude::*;
use colorsys::Rgb;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashMap;
use std::str::FromStr;
use super::render2d::Render2D;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.5,
            ..Default::default()
        },
        transform: Transform::from_xyz(0., 50., 10.)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

pub fn update_cam(
    mut cam_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, (With<ActionState<Dir>>, Changed<Transform>)>,
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok(player_transform) = player_query.get_single() {
            cam_pos.translation.x = player_transform.translation.x;
            cam_pos.translation.z = player_transform.translation.z;
        }
    }
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut col_ents: ResMut<ColEntities>,
) {
    for col_ev in ev_unload.read() {
        if let Some(ent) = col_ents.0.remove(&col_ev.0) {
            commands.entity(ent).despawn();
        }
    }
}

pub fn process_chunk_changes(
    loaded_cols: Res<LoadedCols>,
    mut commands: Commands,
    mut blocs: ResMut<Blocs>, 
    im_query: Query<&Handle<Image>>,
    mut images: ResMut<Assets<Image>>,
    mut col_ents: ResMut<ColEntities>,
    soil_color: Res<SoilColor>,
) {
    if let Some(chunk) = blocs.changes.pop() {
        let col: ColPos = chunk.into();
        if !loaded_cols.in_player_range(col) { return; }
        if let Some(ent) = col_ents.0.get(&col) {
            if let Ok(handle) = im_query.get_component::<Handle<Image>>(*ent) {
                if let Some(image) = images.get_mut(handle) {
                    blocs.update_image(chunk.into(), image, &soil_color);
                }
            } else {
                // the entity is not instanciated yet, we put it back
                blocs.changes.insert(chunk);
            }
        } else {
            let trans = Vec3::new(col.x as f32, 0., col.z as f32) * CHUNK_S1 as f32;
            let ent = commands
                .spawn(SpriteBundle {
                    texture: images.add(blocs.create_image(col, &soil_color)),
                    transform: Transform::from_translation(trans)
                        .looking_at(trans + Vec3::Y, Vec3::Y),
                    ..default()
                })
                .id();
            col_ents.0.insert(col, ent);
        }
    }
}

#[derive(Resource)]
pub struct SoilColor(pub HashMap<Bloc, Rgb>);

impl SoilColor {
    pub fn from_csv(path: &str) -> Result<Self> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut data = HashMap::new();
        for record in reader.records() {
            let record = record?;
            let color = Rgb::from_hex_str(&record[1].trim())?;
            data.insert(Bloc::from_str(&record[0]).unwrap(), color);
        }
        Ok(SoilColor(data))
    }
}

#[derive(Resource)]
pub struct ColEntities(pub HashMap::<ColPos, Entity>);

impl ColEntities {
    pub fn new() -> Self {
        ColEntities(HashMap::new())
    }
}

pub struct Draw2d;

impl Plugin for Draw2d {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoilColor::from_csv("assets/data/soils_color.csv").unwrap())
            .insert_resource(ColEntities::new())
            .add_systems(Startup, setup)
            .add_systems(Update, update_cam)
            .add_systems(Update, on_col_unload)
            .add_systems(Update, process_chunk_changes)
            ;
    }
}