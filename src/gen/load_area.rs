use crate::blocs::{ColPos, Realm, BlocPos};
use bevy::prelude::Query;
use bevy::prelude::*;
use itertools::iproduct;
use std::ops::{Deref, Sub};
use crate::gen::load_cols::LoadedCols;

#[derive(Component, Clone, Copy)]
pub struct LoadArea {
    pub col: ColPos,
    pub dist: u32,
}

impl LoadArea {
    pub fn contains(&self, col: ColPos) -> bool {
        // checks if a chunk is in this Player loaded area (assuming they're in the same realm)
        self.col.dist(col) <= self.dist as i32
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, i32)> {
        let dist = self.dist as i32;
        iproduct!(
            (self.col.x - dist)..=(self.col.x + dist),
            (self.col.z - dist)..=(self.col.z + dist)
        )
    }
}

impl Sub<LoadArea> for LoadArea {
    type Output = Vec<(i32, i32)>;

    fn sub(self, rhs: LoadArea) -> Self::Output {
        if self.col.realm != rhs.col.realm {
            self.iter().collect()
        } else {
            self.iter()
                .filter(|(x, z)| {
                    !rhs.contains(ColPos {
                        realm: self.col.realm,
                        x: *x,
                        z: *z,
                    })
                })
                .collect()
        }
    }
}

pub fn update_load_area(mut query: Query<(&Transform, &Realm, &mut LoadArea)>) {
    for (transform, realm, mut load_area) in query.iter_mut() {
        let col = ColPos::from(BlocPos::from((transform.translation, *realm)));
        // we're checking before modifying to avoid triggering unnecessary Change detection
        if col != load_area.col {
            load_area.col = col;
        }
    }
}

#[derive(Component)]
pub struct LoadAreaOld(LoadArea);

impl Deref for LoadAreaOld {
    type Target = LoadArea;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn load_order(
    mut commands: Commands,
    mut query: Query<(&LoadArea, Option<&mut LoadAreaOld>, Entity), Changed<LoadArea>>,
    mut world: ResMut<LoadedCols>,
) {
    for (load_area, load_area_old_opt, entity) in query.iter_mut() {
        // compute the columns to load and unload & update old load area
        let load_area_clone = LoadAreaOld(load_area.clone());
        if let Some(mut load_area_old) = load_area_old_opt {
            world.register(
                *load_area - **load_area_old,
                load_area.col.realm,
                entity.index(),
            );
            world.unregister(
                **load_area_old - *load_area,
                load_area_old.col.realm,
                entity.index()
            );
            *load_area_old = load_area_clone;
        } else {
            commands.entity(entity).insert(load_area_clone);
            world.register(load_area.iter().collect(), load_area.col.realm, entity.index());
        }
    }
}
