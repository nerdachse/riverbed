use std::f32::consts::PI;
use bevy::prelude::*;
use bevy_atmosphere::{prelude::{AtmospherePlugin, AtmosphereCamera, Nishita, AtmosphereModel}, system_param::AtmosphereMut};
use crate::game3d::camera::{CameraSpawn, FpsCam};
const DAY_LENGTH_MINUTES: f32 = 0.2;
const C: f32 = DAY_LENGTH_MINUTES*120.*PI;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
struct CycleTimer(Timer);


#[derive(Component)]
struct Sun;

fn spawn_sun(mut commands: Commands, cam_query: Query<Entity, With<FpsCam>>) {
    let cam = cam_query.get_single().unwrap();
    commands.entity(cam).insert(AtmosphereCamera::default());
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            ..default()
        },
        ..default()
    }).insert(Sun);
}

// We can edit the Atmosphere resource and it will be updated automatically
fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let t = 0.1 + time.elapsed_seconds_wrapped() / C;
        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
            light_trans.rotation = Quat::from_rotation_x(-t);
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
        }
    }
}

pub struct SkyPlugin;


impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app        
            .insert_resource(AtmosphereModel::new(Nishita {
                rayleigh_coefficient: Vec3::new(5.5e-6, 4.0e-6, 22.4e-6),
                mie_coefficient: 15e-6,
                ..default()
            }))
            .insert_resource(CycleTimer(Timer::new(
                 // Update our atmosphere every 500ms
                bevy::utils::Duration::from_millis(500),
                TimerMode::Repeating,
            )))
            .add_plugins(AtmospherePlugin)
            .add_systems(Startup, spawn_sun.after(CameraSpawn))
            .add_systems(Update, daylight_cycle)
            ;
    }
}