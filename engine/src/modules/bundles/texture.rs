//! `BundleTexture` (3D): downloads a bundle on demand, then puts one of its images on the
//! object's material. Bundles other than the preloaded ones load while the game runs. Part of
//! the engine (bundling is), installed by `launch_project`.

use bevy::prelude::*;

use super::{BundleReady, LoadBundle, load_image};

#[derive(Component, Clone, Debug, PartialEq)]
pub struct BundleTexture {
    pub bundle: &'static str,
    pub texture: &'static str,
}

/// Installed by `launch_project`.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, (request, apply));
}

fn request(
    added: Query<&BundleTexture, Added<BundleTexture>>,
    mut load: MessageWriter<LoadBundle>,
) {
    for wanted in &added {
        load.write(LoadBundle(wanted.bundle));
    }
}

fn apply(
    mut commands: Commands,
    mut ready: MessageReader<BundleReady>,
    objects: Query<(Entity, &BundleTexture, &MeshMaterial3d<StandardMaterial>)>,
) {
    for BundleReady(id) in ready.read() {
        for (entity, wanted, material) in &objects {
            if wanted.bundle != *id {
                continue;
            }
            let (material, path) = (material.0.clone(), wanted.texture);
            commands.queue(move |world: &mut World| {
                let texture = load_image(world, path);
                if let Some(material) = world
                    .resource_mut::<Assets<StandardMaterial>>()
                    .get_mut(&material)
                {
                    material.base_color_texture = Some(texture);
                }
            });
            commands.entity(entity).remove::<BundleTexture>();
        }
    }
}
