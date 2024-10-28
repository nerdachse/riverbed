use bevy::{audio::{PlaybackMode, SpatialScale}, prelude::*};
use rand::Rng;
use crate::{agents::{BlockPlaced, Furnace}, items::LitFurnace};
const RAND_AMPLITUDE: f32 = 0.3;

pub struct EffectSoundPlugin;

impl Plugin for EffectSoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, setup_effect_sounds)
            .add_systems(Update, setup_furnace_cd)
            .add_systems(Update, furnace_sounds)
            .observe(on_block_placed)
            ;
    }
}

#[derive(Resource)]
pub struct EffectSounds {
    item_get: Handle<AudioSource>,
    fire_crackle: Handle<AudioSource>,
    flame: Handle<AudioSource>,
    block_placed: Handle<AudioSource>,
}

fn setup_effect_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(EffectSounds {
        item_get: asset_server.load("sounds/effects/pop.ogg"),
        fire_crackle: asset_server.load("sounds/effects/tt.ogg"),
        flame: asset_server.load("sounds/effects/flame.ogg"),
        block_placed: asset_server.load("sounds/effects/p.ogg"),
    });
}

#[derive(Event)]
pub struct ItemGet;

pub fn on_item_get(_: Trigger<ItemGet>, mut commands: Commands, effect_sounds: Res<EffectSounds>) {
    commands.spawn(AudioSourceBundle {
        source: effect_sounds.item_get.clone_weak(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..Default::default()
        }
    });
}

pub fn on_block_placed(
    block_placed: Trigger<BlockPlaced>, 
    mut commands: Commands, 
    effect_sounds: Res<EffectSounds>
) {
    commands
        .spawn(SpatialBundle::from_transform(Transform::from_translation(block_placed.event().0.into())))
        .insert(AudioSourceBundle {
            source: effect_sounds.block_placed.clone_weak(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                spatial: true,
                spatial_scale: Some(SpatialScale::new(0.2)),
                ..Default::default()
            }
        });
}


#[derive(Component)]
struct FireCrackleCD(pub f32);

#[derive(Component)]
struct FlameCD(pub f32);

fn setup_furnace_cd(
    mut commands: Commands,
    furnace_query: Query<Entity, (With<LitFurnace>, Without<FlameCD>)>
) {
    for furnace in furnace_query.iter() {
        commands.entity(furnace).insert((FireCrackleCD(0.), FlameCD(0.)));
    }
}

fn furnace_sounds(
    mut commands: Commands, 
    effect_sounds: Res<EffectSounds>,
    time: Res<Time>, 
    mut furnace_query: Query<(&Furnace, &mut FireCrackleCD, &mut FlameCD), With<LitFurnace>>
) {
    for (furnace, mut fire_crackle_cd, mut flame_cd) in furnace_query.iter_mut() {
        fire_crackle_cd.0 -= time.delta_seconds();
        if fire_crackle_cd.0 <= 0. {
            commands.spawn(SpatialBundle::from_transform(Transform::from_translation(furnace.block_pos.into())))
            .insert(AudioSourceBundle {
                source: effect_sounds.fire_crackle.clone(),
                settings: PlaybackSettings { 
                    mode: PlaybackMode::Despawn, 
                    spatial: true,
                    spatial_scale: Some(SpatialScale::new(0.5)),
                    speed: 1.+((rand::thread_rng().gen::<f32>()-0.5)*RAND_AMPLITUDE),
                    ..Default::default()
                }
            });
            fire_crackle_cd.0 = 0.2+rand::thread_rng().gen::<f32>();
        }
        flame_cd.0 -= time.delta_seconds();
        if flame_cd.0 <= 0. {
            commands.spawn(SpatialBundle::from_transform(Transform::from_translation(furnace.block_pos.into())))
            .insert(AudioSourceBundle {
                source: effect_sounds.flame.clone(),
                settings: PlaybackSettings { 
                    mode: PlaybackMode::Despawn, 
                    spatial: true,
                    spatial_scale: Some(SpatialScale::new(0.2)),
                    speed: 1.+((rand::thread_rng().gen::<f32>()-0.5)*RAND_AMPLITUDE),
                    ..Default::default()
                }
            });
            flame_cd.0 = 2.;
        }
    }
}