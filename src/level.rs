use avian3d::prelude::{Collider, RigidBody};
use bevy::{math::Affine2, prelude::*};

pub struct LevelPlugin;

#[derive(Component)]
pub struct Geometry;

#[derive(Resource)]
pub struct TextureAssets {
    pub prototype_textures: Vec<Handle<Image>>,
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_assets, create_level).chain());
        app.insert_resource(AmbientLight {
            brightness: 700.0,
            ..default()
        });
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let prototype_textures = vec![
        asset_server.load("textures/prototype_0.png"),
        asset_server.load("textures/prototype_1.png"),
        asset_server.load("textures/prototype_2.png"),
        asset_server.load("textures/prototype_3.png"),
        asset_server.load("textures/prototype_4.png"),
        asset_server.load("textures/prototype_5.png"),
        asset_server.load("textures/prototype_6.png"),
        asset_server.load("textures/prototype_7.png"),
    ];

    commands.insert_resource(TextureAssets { prototype_textures });
}

fn create_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_assets: Res<TextureAssets>,
) {
    let map_scaler: f32 = 5.;

    let prototype_textures = level_assets.prototype_textures.clone();

    if prototype_textures.is_empty() {
        warn!("No prototype textures found. Skipping texture assignment.");
        return;
    }

    // --- Ground ---
    let ground_size = Vec3::new(20.0, 0.5, 20.0) * map_scaler;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(ground_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            // use the first texture in the prototype textures
            base_color_texture: Some(prototype_textures[7].clone()),
            uv_transform: Affine2::from_scale(Vec2::new(ground_size.x, ground_size.z) / 5.0),
            ..Default::default()
        })),
        // Collider::cuboid(ground_size.x, ground_size.y, ground_size.z),
        // RigidBody::Static,
        Geometry,
        Name::new("Ground"),
    ));

    // --- Wall ---
    let wall_size = Vec3::new(0.1, 2.0, 2.0) * map_scaler;
    commands.spawn((
        Transform::from_xyz(5.0, wall_size.y / 2.0, 0.0),
        Mesh3d(meshes.add(Cuboid::from_size(wall_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(prototype_textures[3 * 13].clone()),
            uv_transform: Affine2::from_scale(Vec2::new(wall_size.y, wall_size.z) / 5.0),
            ..Default::default()
        })),
        // Collider::cuboid(wall_size.x, wall_size.y, wall_size.z),
        // RigidBody::Static,
        Geometry,
        Name::new("Wall"),
    ));

    // --- Angled Wall ---
    let angled_wall_size = Vec3::new(0.1, 2.0, 3.0) * map_scaler;
    let wall_angle = 45.0_f32.to_radians();
    commands.spawn((
        // Position to connect with the first wall at an angle
        Transform::from_xyz(5.0, angled_wall_size.y / 2.0, -1.0)
            .with_rotation(Quat::from_rotation_y(wall_angle)),
        Mesh3d(meshes.add(Cuboid::from_size(angled_wall_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(prototype_textures[3 * 13].clone()),
            uv_transform: Affine2::from_scale(
                Vec2::new(angled_wall_size.y, angled_wall_size.z) / 5.0,
            ),
            ..Default::default()
        })),
        // Collider::cuboid(angled_wall_size.x, angled_wall_size.y, angled_wall_size.z),
        // RigidBody::Static,
        Geometry,
        Name::new("AngledWall"),
    ));

    // --- Jump Box ---
    let box_size = Vec3::new(1.0, 1.0, 1.0) * map_scaler;
    commands.spawn((
        Transform::from_xyz(-5.0, box_size.y / 2.0, 3.0),
        Mesh3d(meshes.add(Cuboid::from_size(box_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(prototype_textures[4 * 13 + 3].clone()),
            uv_transform: Affine2::from_scale(Vec2::new(box_size.x, box_size.z) / 5.0),
            ..Default::default()
        })),
        // Collider::cuboid(box_size.x, box_size.y, box_size.z),
        // RigidBody::Static,
        Geometry,
        Name::new("JumpBox"),
    ));

    // --- Slope - Gentle ---

    // a long gentle slope
    let gentle_slope_size = Vec3::new(5.0, 0.5, 15.0);
    let gentle_slope_angle = 10.0_f32.to_radians();
    commands.spawn((
        Transform::from_xyz(-8.0, gentle_slope_size.y / 2.0 + 0.75, -3.0)
            .with_rotation(Quat::from_rotation_x(gentle_slope_angle)),
        Mesh3d(meshes.add(Cuboid::from_size(gentle_slope_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(prototype_textures[5 * 13].clone()),
            uv_transform: Affine2::from_scale(
                Vec2::new(gentle_slope_size.x, gentle_slope_size.z) / 5.0,
            ),
            ..Default::default()
        })),
        // Collider::cuboid(
        //     gentle_slope_size.x,
        //     gentle_slope_size.y,
        //     gentle_slope_size.z,
        // ),
        // RigidBody::Static,
        Geometry,
        Name::new("GentleSlope"),
    ));

    // --- Slope - Medium Steep ---
    let steep_slope_size = Vec3::new(5.0, 0.5, 10.0);
    let steep_slope_angle = 30.0_f32.to_radians();

    commands.spawn((
        Transform::from_xyz(-8.0, steep_slope_size.y / 2.0 + 0.75, -10.0)
            .with_rotation(Quat::from_rotation_x(steep_slope_angle)),
        Mesh3d(meshes.add(Cuboid::from_size(steep_slope_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(prototype_textures[4 * 13].clone()),
            uv_transform: Affine2::from_scale(
                Vec2::new(steep_slope_size.x, steep_slope_size.z) / 5.0,
            ),
            ..Default::default()
        })),
        // Collider::cuboid(steep_slope_size.x, steep_slope_size.y, steep_slope_size.z),
        // RigidBody::Static,
        Geometry,
        Name::new("SteepSlope"),
    ));

    // --- Steps ---
    let step_base_size = Vec3::new(1.0, 0.15, 4.0);
    let step_colors = [
        Color::srgb_u8(200, 200, 100), // Yellow-ish
        Color::srgb_u8(180, 180, 100),
        Color::srgb_u8(160, 160, 100),
        Color::srgb_u8(140, 140, 100),
    ];

    for i in 0..4 {
        let height_multiplier = (i as f32) + 1.0;
        let step_height = 0.12 * height_multiplier;
        let position_x = -8.0 + (i as f32 * step_base_size.x);

        commands.spawn((
            Transform::from_xyz(position_x, step_height + 1.25, 8.0),
            Mesh3d(meshes.add(Cuboid::new(step_base_size.x, step_height, step_base_size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(prototype_textures[9].clone()),
                uv_transform: Affine2::from_scale(
                    Vec2::new(step_base_size.x, step_base_size.z) / 5.0,
                ),
                ..Default::default()
            })),
            // Collider::cuboid(step_base_size.x, step_height, step_base_size.z),
            // RigidBody::Static,
            Geometry,
            Name::new(format!("Step_{}", i + 1)),
        ));
    }
}
