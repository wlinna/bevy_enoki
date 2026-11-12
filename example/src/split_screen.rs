/// ----------------------------------------------
/// split screen example
/// particles seen by 1, 2 or 4 cameras; the layout cycles every second (or on F1)
/// ----------------------------------------------
use bevy::{
    camera::{Hdr, Viewport},
    image::ImageSamplerDescriptor,
    input::common_conditions::input_just_pressed,
    post_process::bloom::Bloom,
    prelude::*,
    time::common_conditions::on_timer,
    window::WindowResized,
};
use bevy_enoki::{prelude::*, EnokiPlugin};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins(EnokiPlugin)
        .init_resource::<CameraLayout>()
        .add_systems(Startup, (spawn_cameras, setup))
        .add_systems(
            Update,
            (
                (cycle_camera_layout, spawn_cameras).chain().run_if(
                    input_just_pressed(KeyCode::F1).or_else(on_timer(Duration::from_secs(1))),
                ),
                set_camera_viewports
                    .run_if(on_message::<WindowResized>.or_else(resource_changed::<CameraLayout>))
                    .after(spawn_cameras),
            ),
        )
        .run();
}

/// Where a camera sits in the window, in halves.
#[derive(Component, Default)]
struct CameraSlot {
    right: bool,
    bottom: bool,
    half_width: bool,
    half_height: bool,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
enum CameraLayout {
    #[default]
    Single,
    VerticalSplit,
    QuadSplit,
}

fn cycle_camera_layout(mut layout: ResMut<CameraLayout>) {
    *layout = match *layout {
        CameraLayout::Single => CameraLayout::VerticalSplit,
        CameraLayout::VerticalSplit => CameraLayout::QuadSplit,
        CameraLayout::QuadSplit => CameraLayout::Single,
    };
    info!("camera layout: {:?}", *layout);
}

fn spawn_cameras(
    mut cmd: Commands,
    cameras: Query<Entity, With<CameraSlot>>,
    layout: Res<CameraLayout>,
) {
    for camera in &cameras {
        cmd.entity(camera).despawn();
    }
    let slots = match *layout {
        CameraLayout::Single => vec![CameraSlot::default()],
        CameraLayout::VerticalSplit => [false, true]
            .map(|right| CameraSlot {
                right,
                half_width: true,
                ..default()
            })
            .into(),
        CameraLayout::QuadSplit => [(false, false), (true, false), (false, true), (true, true)]
            .map(|(right, bottom)| CameraSlot {
                right,
                bottom,
                half_width: true,
                half_height: true,
            })
            .into(),
    };
    for (order, slot) in slots.into_iter().enumerate() {
        cmd.spawn((
            Camera2d,
            Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                order: order as isize,
                ..default()
            },
            Hdr,
            Bloom {
                intensity: 0.1,
                ..default()
            },
            slot,
        ));
    }
}

fn set_camera_viewports(
    windows: Query<&Window>,
    mut cameras: Query<(&CameraSlot, &mut Camera)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let size = window.physical_size();
    let half = size / 2;
    for (slot, mut camera) in &mut cameras {
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(
                if slot.right { half.x } else { 0 },
                if slot.bottom { half.y } else { 0 },
            ),
            physical_size: UVec2::new(
                if slot.half_width { half.x } else { size.x },
                if slot.half_height { half.y } else { size.y },
            ),
            ..default()
        });
    }
}

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    server: Res<AssetServer>,
) {
    let material_handle = materials.add(SpriteParticle2dMaterial::from_texture(
        server.load("enoki.png"),
    ));
    cmd.spawn((
        ParticleSpawner(material_handle),
        ParticleEffectHandle(server.load("base.particle.ron")),
    ));
}
