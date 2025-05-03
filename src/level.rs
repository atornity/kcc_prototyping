use std::f32::consts::PI;

use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    animation::{AnimationTarget, AnimationTargetId, animated_field},
    asset::AssetServerMode,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    math::Affine2,
    prelude::*,
};

pub struct LevelPlugin;

#[derive(Component)]
pub struct Geometry;

#[derive(Resource)]
pub struct TextureAssets {
    pub prototype_textures: Vec<Handle<Image>>,
}

#[derive(Resource, Default)]
pub struct LoadingAssets {
    pub handles: Vec<UntypedHandle>,
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_assets, create_level).chain());
        app.insert_resource(LoadingAssets::default());
        app.insert_resource(TextureAssets {
            prototype_textures: vec![],
        });
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
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let mut prototype_textures = vec![];

    // there is 6 colors where each color has 13 textures
    // we will load all of them for now: where the format is: asset_server.load("materials/prototyping/blocks/Dark/texture_01.png")
    // the colors are Dark, Light, Green, Red, Orange, Purple
    let colors = ["Dark", "Light", "Green", "Red", "Orange", "Purple"];
    for color in colors {
        for i in 1..=13 {
            let texture_path = format!("textures/{}/texture_{:02}.png", color, i);
            let handle = asset_server.load_with_settings(texture_path, |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        // rewriting mode to repeat image,
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        address_mode_w: ImageAddressMode::Repeat,
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Linear,
                        mipmap_filter: ImageFilterMode::Linear,
                        lod_min_clamp: 0.0,
                        lod_max_clamp: 32.0,
                        anisotropy_clamp: 1,
                        ..Default::default()
                    }),
                    ..default()
                }
            });
            prototype_textures.push(handle.clone());
            loading_assets.handles.push(handle.untyped());
        }
    }

    commands.insert_resource(TextureAssets { prototype_textures });
}

// fn create_level(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     level_assets: Res<TextureAssets>,
// ) {
//     let map_scaler: f32 = 5.;
//
//     let prototype_textures = level_assets.prototype_textures.clone();
//
//     if prototype_textures.is_empty() {
//         warn!("No prototype textures found. Skipping texture assignment.");
//         return;
//     }
//
//     // --- Ground ---
//     let ground_size = Vec3::new(20.0, 0.5, 20.0) * map_scaler;
//     commands.spawn((
//         Mesh3d(meshes.add(Cuboid::from_size(ground_size))),
//         MeshMaterial3d(materials.add(StandardMaterial {
//             // use the first texture in the prototype textures
//             base_color_texture: Some(prototype_textures[7].clone()),
//             uv_transform: Affine2::from_scale(Vec2::new(ground_size.x, ground_size.z) / 5.0),
//             ..Default::default()
//         })),
//         Collider::cuboid(ground_size.x, ground_size.y, ground_size.z),
//         RigidBody::Static,
//         Geometry,
//         Name::new("Ground"),
//     ));
//
//     // --- Wall ---
//     let wall_size = Vec3::new(0.1, 2.0, 2.0) * map_scaler;
//     commands.spawn((
//         Transform::from_xyz(5.0, wall_size.y / 2.0, 0.0),
//         Mesh3d(meshes.add(Cuboid::from_size(wall_size))),
//         MeshMaterial3d(materials.add(StandardMaterial {
//             base_color_texture: Some(prototype_textures[3 * 13].clone()),
//             uv_transform: Affine2::from_scale(Vec2::new(wall_size.y, wall_size.z) / 5.0),
//             ..Default::default()
//         })),
//         Collider::cuboid(wall_size.x, wall_size.y, wall_size.z),
//         RigidBody::Static,
//         Geometry,
//         Name::new("Wall"),
//     ));
//
//     // --- Angled Wall ---
//     let angled_wall_size = Vec3::new(0.1, 2.0, 3.0) * map_scaler;
//     let wall_angle = 45.0_f32.to_radians();
//     commands.spawn((
//         // Position to connect with the first wall at an angle
//         Transform::from_xyz(5.0, angled_wall_size.y / 2.0, -1.0)
//             .with_rotation(Quat::from_rotation_y(wall_angle)),
//         Mesh3d(meshes.add(Cuboid::from_size(angled_wall_size))),
//         MeshMaterial3d(materials.add(StandardMaterial {
//             base_color_texture: Some(prototype_textures[3 * 13].clone()),
//             uv_transform: Affine2::from_scale(
//                 Vec2::new(angled_wall_size.y, angled_wall_size.z) / 5.0,
//             ),
//             ..Default::default()
//         })),
//         Collider::cuboid(angled_wall_size.x, angled_wall_size.y, angled_wall_size.z),
//         RigidBody::Static,
//         Geometry,
//         Name::new("AngledWall"),
//     ));
//
//     // --- Jump Box ---
//     let box_size = Vec3::new(1.0, 1.0, 1.0) * map_scaler;
//     commands.spawn((
//         Transform::from_xyz(-5.0, box_size.y / 2.0, 3.0),
//         Mesh3d(meshes.add(Cuboid::from_size(box_size))),
//         MeshMaterial3d(materials.add(StandardMaterial {
//             base_color_texture: Some(prototype_textures[4 * 13 + 3].clone()),
//             uv_transform: Affine2::from_scale(Vec2::new(box_size.x, box_size.z) / 5.0),
//             ..Default::default()
//         })),
//         Collider::cuboid(box_size.x, box_size.y, box_size.z),
//         RigidBody::Static,
//         Geometry,
//         Name::new("JumpBox"),
//     ));
//
//     // --- Slope - Gentle ---
//
//     // a long gentle slope
//     let gentle_slope_size = Vec3::new(5.0, 0.5, 15.0);
//     let gentle_slope_angle = 10.0_f32.to_radians();
//     commands.spawn((
//         Transform::from_xyz(-8.0, gentle_slope_size.y / 2.0 + 0.75, -3.0)
//             .with_rotation(Quat::from_rotation_x(gentle_slope_angle)),
//         Mesh3d(meshes.add(Cuboid::from_size(gentle_slope_size))),
//         MeshMaterial3d(materials.add(StandardMaterial {
//             base_color_texture: Some(prototype_textures[5 * 13].clone()),
//             uv_transform: Affine2::from_scale(
//                 Vec2::new(gentle_slope_size.x, gentle_slope_size.z) / 5.0,
//             ),
//             ..Default::default()
//         })),
//         Collider::cuboid(
//             gentle_slope_size.x,
//             gentle_slope_size.y,
//             gentle_slope_size.z,
//         ),
//         RigidBody::Static,
//         Geometry,
//         Name::new("GentleSlope"),
//     ));
//
//     // --- Slope - Medium Steep ---
//     let steep_slope_size = Vec3::new(5.0, 0.5, 10.0);
//     let steep_slope_angle = 30.0_f32.to_radians();
//
//     commands.spawn((
//         Transform::from_xyz(-8.0, steep_slope_size.y / 2.0 + 0.75, -10.0)
//             .with_rotation(Quat::from_rotation_x(steep_slope_angle)),
//         Mesh3d(meshes.add(Cuboid::from_size(steep_slope_size))),
//         MeshMaterial3d(materials.add(StandardMaterial {
//             base_color_texture: Some(prototype_textures[4 * 13].clone()),
//             uv_transform: Affine2::from_scale(
//                 Vec2::new(steep_slope_size.x, steep_slope_size.z) / 5.0,
//             ),
//             ..Default::default()
//         })),
//         Collider::cuboid(steep_slope_size.x, steep_slope_size.y, steep_slope_size.z),
//         RigidBody::Static,
//         Geometry,
//         Name::new("SteepSlope"),
//     ));
//
//     // --- Steps ---
//     let step_base_size = Vec3::new(1.0, 0.15, 4.0);
//     let step_colors = [
//         Color::srgb_u8(200, 200, 100), // Yellow-ish
//         Color::srgb_u8(180, 180, 100),
//         Color::srgb_u8(160, 160, 100),
//         Color::srgb_u8(140, 140, 100),
//     ];
//
//     for i in 0..4 {
//         let height_multiplier = (i as f32) + 1.0;
//         let step_height = 0.12 * height_multiplier;
//         let position_x = -8.0 + (i as f32 * step_base_size.x);
//
//         commands.spawn((
//             Transform::from_xyz(position_x, step_height + 1.25, 8.0),
//             Mesh3d(meshes.add(Cuboid::new(step_base_size.x, step_height, step_base_size.z))),
//             MeshMaterial3d(materials.add(StandardMaterial {
//                 base_color_texture: Some(prototype_textures[9].clone()),
//                 uv_transform: Affine2::from_scale(
//                     Vec2::new(step_base_size.x, step_base_size.z) / 5.0,
//                 ),
//                 ..Default::default()
//             })),
//             Collider::cuboid(step_base_size.x, step_height, step_base_size.z),
//             RigidBody::Static,
//             Geometry,
//             Name::new(format!("Step_{}", i + 1)),
//         ));
//     }
// }
//

// --- Configuration Constants ---
const MAP_SCALER: f32 = 1.0;
const OBJECT_SPACING: f32 = 15.0 * MAP_SCALER;
const BASE_Y: f32 = 0.0;
const UV_TILE_FACTOR: f32 = 5.0; // Controls texture repetition density

const GROUND_WIDTH: f32 = 150.0;
const GROUND_HEIGHT: f32 = 1.0;
const GROUND_DEPTH: f32 = 50.0;
const WALL_HEIGHT: f32 = 4.0;
const WALL_THICKNESS: f32 = 0.2;
const WALL_SEGMENT_LENGTH: f32 = 5.0;

const SMALL_STEP_HEIGHT: f32 = 0.15;
const SMALL_STEP_DEPTH: f32 = 0.3;
const SMALL_STEP_WIDTH: f32 = 3.0;
const NUM_SMALL_STEPS: i32 = 6;

const LARGE_STEP_HEIGHT: f32 = 0.5;
const LARGE_STEP_DEPTH: f32 = 0.8;
const LARGE_STEP_WIDTH: f32 = 3.0;
const NUM_LARGE_STEPS: i32 = 4;

const RAMP_WIDTH: f32 = 4.0;
const RAMP_LENGTH: f32 = 8.0;
const RAMP_THICKNESS: f32 = 0.2;
const SHALLOW_RAMP_ANGLE: f32 = 15.0; // Degrees
const STEEP_RAMP_ANGLE: f32 = 40.0; // Degrees

const LOW_CEILING_HEIGHT: f32 = 1.5;
const CEILING_BLOCK_WIDTH: f32 = 5.0;
const CEILING_BLOCK_HEIGHT: f32 = 0.2;
const CEILING_BLOCK_DEPTH: f32 = 5.0;
const ANGLED_CEILING_ANGLE: f32 = -20.0; // Degrees

const CREVICE_WIDTH: f32 = 0.5;
const CREVICE_WALL_ANGLE: f32 = 60.0; // Degrees
const CREVICE_LENGTH: f32 = 5.0;
const CREVICE_WALL_THICKNESS: f32 = 0.2;
const CREVICE_WALL_HEIGHT: f32 = 5.0;

const MOVING_PLATFORM_WIDTH: f32 = 3.0;
const MOVING_PLATFORM_HEIGHT: f32 = 0.3;
const MOVING_PLATFORM_DEPTH: f32 = 3.0;
const PLATFORM_VERTICAL_DISTANCE: f32 = 5.0;
const PLATFORM_HORIZONTAL_DISTANCE: f32 = 6.0;
const PLATFORM_ROTATION_ANGLE: f32 = PI; // 180 degrees
const PLATFORM_ANIMATION_DURATION: f32 = 3.0; // Seconds for one way

// Texture Indices (adjust based on your `TextureAssets`)
const TEX_GROUND: usize = 7;
const TEX_WALL: usize = 3 * 13;
const TEX_STEP: usize = 9;
const TEX_RAMP_SHALLOW: usize = 5 * 13;
const TEX_RAMP_STEEP: usize = 4 * 13;
const TEX_CEILING: usize = 2 * 13;
const TEX_PLATFORM: usize = 4 * 13 + 3;
const TEX_DEFAULT: usize = 0; // Fallback texture index
// --- Helper Functions ---

/// Calculates UV scaling based on object size to maintain texture density.
fn calculate_uv_scale(object_size: Vec3, tile_factor: f32) -> Affine2 {
    let mut dims = [
        object_size.x.abs(),
        object_size.y.abs(),
        object_size.z.abs(),
    ];
    dims.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    Affine2::from_scale(Vec2::new(dims[0], dims[1]) / tile_factor)
}

/// Creates a StandardMaterial with specific texture and UV transform, adds it to assets.
/// Returns handle to the created material or a fallback if texture index is invalid.
fn create_material_with_uv(
    texture_index: usize,
    object_size: Vec3, // Needed for UV calculation
    uv_tile_factor: f32,
    level_assets: &Res<TextureAssets>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    fallback_material_handle: &Handle<StandardMaterial>, // Pass in the pre-made fallback
) -> Handle<StandardMaterial> {
    match level_assets.prototype_textures.get(texture_index) {
        Some(texture_handle) => {
            // Calculate UV transform for this specific material instance
            let uv_transform = calculate_uv_scale(object_size, uv_tile_factor);

            // Create the material with texture and UV transform
            materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle.clone()),
                uv_transform, // Apply the calculated transform here
                perceptual_roughness: 0.7,
                metallic: 0.1,
                ..default()
            })
        }
        None => {
            // Texture index invalid or assets empty, return the fallback handle
            if level_assets.prototype_textures.is_empty() {
                warn_once!("TextureAssets resource is empty. Using fallback material.");
            } else {
                warn_once!(
                    "Texture index {} is out of bounds (max is {}). Using fallback material.",
                    texture_index,
                    level_assets.prototype_textures.len().saturating_sub(1) // Prevent underflow if len is 0
                );
            }
            fallback_material_handle.clone()
        }
    }
}

// --- Level Creation System ---

pub fn create_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>, // Still needed to add materials
    level_assets: Res<TextureAssets>,
    // Animation Resources
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut current_x_offset = -(GROUND_WIDTH * MAP_SCALER) / 2.0 + OBJECT_SPACING / 2.0;

    // --- Fallback Material Handle ---
    let fallback_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(150, 140, 130),
        perceptual_roughness: 0.8,
        metallic: 0.1,
        ..default()
    });

    // --- Ground ---
    let ground_size = Vec3::new(GROUND_WIDTH, GROUND_HEIGHT, GROUND_DEPTH) * MAP_SCALER;
    let ground_pos = Vec3::new(0.0, BASE_Y - ground_size.y / 2.0, 0.0);
    let ground_material = create_material_with_uv(
        TEX_GROUND,
        ground_size,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(ground_size))), // Use Mesh3d::new in newer Bevy? Check docs if this fails.
        MeshMaterial3d(ground_material),                    // Handle<StandardMaterial> goes here
        Transform::from_translation(ground_pos),
        // Physics and other components:
        RigidBody::Static,
        Collider::cuboid(
            ground_size.x / 2.0,
            ground_size.y / 2.0,
            ground_size.z / 2.0,
        ),
        Geometry,
        Name::new("Ground"),
        // VisibilityBundle::default(), // Might be needed for rendering
    ));

    // --- Small Steps ---
    let section_center_x = current_x_offset;
    let step_start_y = BASE_Y;
    let step_start_z = -5.0 * MAP_SCALER;
    for i in 0..NUM_SMALL_STEPS {
        let step_size = Vec3::new(
            SMALL_STEP_WIDTH * MAP_SCALER,
            SMALL_STEP_HEIGHT * MAP_SCALER,
            SMALL_STEP_DEPTH * MAP_SCALER,
        );
        let x_pos = section_center_x;
        let y_pos = step_start_y + (i as f32 + 0.5) * step_size.y;
        let z_pos = step_start_z + (i as f32 + 0.5) * step_size.z;

        let step_material = create_material_with_uv(
            TEX_STEP,
            step_size,
            UV_TILE_FACTOR,
            &level_assets,
            &mut materials,
            &fallback_material_handle,
        );
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(step_size))),
            MeshMaterial3d(step_material),
            Transform::from_xyz(x_pos, y_pos, z_pos),
            RigidBody::Static,
            Collider::cuboid(step_size.x / 2.0, step_size.y / 2.0, step_size.z / 2.0),
            Geometry,
            Name::new(format!("SmallStep_{}", i + 1)),
            // VisibilityBundle::default(),
        ));
    }
    current_x_offset += OBJECT_SPACING;

    // --- Large Steps ---
    let section_center_x = current_x_offset;
    let step_start_y = BASE_Y;
    let step_start_z = -5.0 * MAP_SCALER;
    for i in 0..NUM_LARGE_STEPS {
        let step_size = Vec3::new(
            LARGE_STEP_WIDTH * MAP_SCALER,
            LARGE_STEP_HEIGHT * MAP_SCALER,
            LARGE_STEP_DEPTH * MAP_SCALER,
        );
        let x_pos = section_center_x;
        let y_pos = step_start_y + (i as f32 + 0.5) * step_size.y;
        let z_pos = step_start_z + (i as f32 + 0.5) * step_size.z;

        let step_material = create_material_with_uv(
            TEX_STEP,
            step_size,
            UV_TILE_FACTOR,
            &level_assets,
            &mut materials,
            &fallback_material_handle,
        ); // Can use a different index if desired
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(step_size))),
            MeshMaterial3d(step_material),
            Transform::from_xyz(x_pos, y_pos, z_pos),
            RigidBody::Static,
            Collider::cuboid(step_size.x / 2.0, step_size.y / 2.0, step_size.z / 2.0),
            Geometry,
            Name::new(format!("LargeStep_{}", i + 1)),
            // VisibilityBundle::default(),
        ));
    }
    current_x_offset += OBJECT_SPACING;

    // --- Shallow Ramp ---
    let section_center_x = current_x_offset;
    let ramp_size = Vec3::new(
        RAMP_WIDTH * MAP_SCALER,
        RAMP_THICKNESS * MAP_SCALER,
        RAMP_LENGTH * MAP_SCALER,
    );
    let angle_rad = SHALLOW_RAMP_ANGLE.to_radians();
    let y_offset =
        (ramp_size.z / 2.0) * angle_rad.sin() + BASE_Y + ramp_size.y / 2.0 * angle_rad.cos();
    let z_pos = 0.0;
    let transform = Transform::from_xyz(section_center_x, y_offset, z_pos)
        .with_rotation(Quat::from_rotation_x(-angle_rad));

    let ramp_material = create_material_with_uv(
        TEX_RAMP_SHALLOW,
        ramp_size,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(ramp_size))),
        MeshMaterial3d(ramp_material),
        transform, // Apply calculated transform
        RigidBody::Static,
        Collider::cuboid(ramp_size.x / 2.0, ramp_size.y / 2.0, ramp_size.z / 2.0),
        Geometry,
        Name::new("ShallowRamp"),
        // VisibilityBundle::default(),
    ));
    current_x_offset += OBJECT_SPACING;

    // --- Steep Ramp ---
    // (Similar structure to Shallow Ramp, just change angle and texture index)
    let section_center_x = current_x_offset;
    let ramp_size = Vec3::new(
        RAMP_WIDTH * MAP_SCALER,
        RAMP_THICKNESS * MAP_SCALER,
        RAMP_LENGTH * MAP_SCALER,
    );
    let angle_rad = STEEP_RAMP_ANGLE.to_radians();
    let y_offset =
        (ramp_size.z / 2.0) * angle_rad.sin() + BASE_Y + ramp_size.y / 2.0 * angle_rad.cos();
    let z_pos = 0.0;
    let transform = Transform::from_xyz(section_center_x, y_offset, z_pos)
        .with_rotation(Quat::from_rotation_x(-angle_rad));

    let ramp_material = create_material_with_uv(
        TEX_RAMP_STEEP,
        ramp_size,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(ramp_size))),
        MeshMaterial3d(ramp_material),
        transform,
        RigidBody::Static,
        Collider::cuboid(ramp_size.x / 2.0, ramp_size.y / 2.0, ramp_size.z / 2.0),
        Geometry,
        Name::new("SteepRamp"),
        // VisibilityBundle::default(),
    ));
    current_x_offset += OBJECT_SPACING;

    // --- Walls with Right Angle Corner ---
    let section_center_x = current_x_offset;
    let corner_z = -5.0 * MAP_SCALER;
    let wall_height_scaled = WALL_HEIGHT * MAP_SCALER;
    let wall_thickness_scaled = WALL_THICKNESS * MAP_SCALER;
    let wall_segment_length_scaled = WALL_SEGMENT_LENGTH * MAP_SCALER;
    let wall_y = BASE_Y + wall_height_scaled / 2.0;

    let wall_size_z = Vec3::new(
        wall_thickness_scaled,
        wall_height_scaled,
        wall_segment_length_scaled,
    );
    let wall_size_x = Vec3::new(
        wall_segment_length_scaled + wall_thickness_scaled,
        wall_height_scaled,
        wall_thickness_scaled,
    );

    let wall_material_z = create_material_with_uv(
        TEX_WALL,
        wall_size_z,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );
    let wall_material_x = create_material_with_uv(
        TEX_WALL,
        wall_size_x,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );

    // Wall 1 (Along Z)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(wall_size_z))),
        MeshMaterial3d(wall_material_z),
        Transform::from_xyz(
            section_center_x - wall_segment_length_scaled / 2.0,
            wall_y,
            corner_z + wall_segment_length_scaled / 2.0,
        ),
        RigidBody::Static,
        Collider::cuboid(
            wall_size_z.x / 2.0,
            wall_size_z.y / 2.0,
            wall_size_z.z / 2.0,
        ),
        Geometry,
        Name::new("CornerWall_Z"), // VisibilityBundle::default(),
    ));

    // Wall 2 (Along X)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(wall_size_x))),
        MeshMaterial3d(wall_material_x),
        Transform::from_xyz(section_center_x, wall_y, corner_z),
        RigidBody::Static,
        Collider::cuboid(
            wall_size_x.x / 2.0,
            wall_size_x.y / 2.0,
            wall_size_x.z / 2.0,
        ),
        Geometry,
        Name::new("CornerWall_X"), // VisibilityBundle::default(),
    ));
    current_x_offset += OBJECT_SPACING;

    // --- Walls with Acute Angle Corner ---
    // (Similar structure, calculate positions and rotations, call create_material_with_uv)
    let section_center_x = current_x_offset;
    let corner_z = -5.0 * MAP_SCALER;
    let acute_angle = 60.0_f32.to_radians();
    let wall_height_scaled = WALL_HEIGHT * MAP_SCALER;
    let wall_thickness_scaled = WALL_THICKNESS * MAP_SCALER;
    let wall_segment_length_scaled = WALL_SEGMENT_LENGTH * MAP_SCALER;
    let wall_y = BASE_Y + wall_height_scaled / 2.0;
    let wall_size = Vec3::new(
        wall_thickness_scaled,
        wall_height_scaled,
        wall_segment_length_scaled,
    );

    let wall_material = create_material_with_uv(
        TEX_WALL,
        wall_size,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );

    // Wall 1 (Straight along Z)
    let wall1_pos = Vec3::new(
        section_center_x,
        wall_y,
        corner_z + wall_segment_length_scaled / 2.0,
    );
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(wall_size))),
        MeshMaterial3d(wall_material.clone()), // Clone handle if reusing same material instance
        Transform::from_translation(wall1_pos),
        RigidBody::Static,
        Collider::cuboid(wall_size.x / 2.0, wall_size.y / 2.0, wall_size.z / 2.0),
        Geometry,
        Name::new("AcuteWall_1"), // VisibilityBundle::default(),
    ));

    // Wall 2 (Angled)
    let rotation = Quat::from_rotation_y(-acute_angle);
    let offset = rotation * Vec3::new(0.0, 0.0, wall_segment_length_scaled / 2.0);
    let wall2_pos = Vec3::new(section_center_x, wall_y, corner_z) + offset;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(wall_size))),
        MeshMaterial3d(wall_material), // Can reuse if cloning above, otherwise create new if needed
        Transform::from_translation(wall2_pos).with_rotation(rotation),
        RigidBody::Static,
        Collider::cuboid(wall_size.x / 2.0, wall_size.y / 2.0, wall_size.z / 2.0),
        Geometry,
        Name::new("AcuteWall_2"), // VisibilityBundle::default(),
    ));
    current_x_offset += OBJECT_SPACING;

    // --- Low Ceiling ---
    let section_center_x = current_x_offset;
    let ceiling_size = Vec3::new(
        CEILING_BLOCK_WIDTH * MAP_SCALER,
        CEILING_BLOCK_HEIGHT * MAP_SCALER,
        CEILING_BLOCK_DEPTH * MAP_SCALER,
    );
    let ceiling_pos = Vec3::new(
        section_center_x,
        BASE_Y + LOW_CEILING_HEIGHT * MAP_SCALER + ceiling_size.y / 2.0,
        0.0,
    );
    let ceiling_material = create_material_with_uv(
        TEX_CEILING,
        ceiling_size,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(ceiling_size))),
        MeshMaterial3d(ceiling_material),
        Transform::from_translation(ceiling_pos),
        RigidBody::Static,
        Collider::cuboid(
            ceiling_size.x / 2.0,
            ceiling_size.y / 2.0,
            ceiling_size.z / 2.0,
        ),
        Geometry,
        Name::new("LowCeiling"), // VisibilityBundle::default(),
    ));
    current_x_offset += OBJECT_SPACING;

    // --- Angled Ceiling ---
    // (Similar structure, just add rotation)
    let section_center_x = current_x_offset;
    let ceiling_size = Vec3::new(
        CEILING_BLOCK_WIDTH * MAP_SCALER,
        CEILING_BLOCK_HEIGHT * MAP_SCALER,
        CEILING_BLOCK_DEPTH * MAP_SCALER,
    );
    let angle_rad = ANGLED_CEILING_ANGLE.to_radians();
    let ceiling_center_y = BASE_Y + (WALL_HEIGHT * MAP_SCALER);
    let transform = Transform::from_xyz(section_center_x, ceiling_center_y, 0.0)
        .with_rotation(Quat::from_rotation_x(angle_rad));

    let ceiling_material = create_material_with_uv(
        TEX_CEILING,
        ceiling_size,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(ceiling_size))),
        MeshMaterial3d(ceiling_material),
        transform,
        RigidBody::Static,
        Collider::cuboid(
            ceiling_size.x / 2.0,
            ceiling_size.y / 2.0,
            ceiling_size.z / 2.0,
        ),
        Geometry,
        Name::new("AngledCeiling"), // VisibilityBundle::default(),
    ));
    current_x_offset += OBJECT_SPACING;

    // --- Crevice / Funnel ---
    // (Spawn two angled walls using the component pattern)
    let section_center_x = current_x_offset;
    let crevice_wall_size = Vec3::new(
        CREVICE_WALL_THICKNESS * MAP_SCALER,
        CREVICE_WALL_HEIGHT * MAP_SCALER,
        CREVICE_LENGTH * MAP_SCALER,
    );
    let angle_rad = CREVICE_WALL_ANGLE.to_radians();
    let horz_offset =
        (CREVICE_WIDTH * MAP_SCALER / 2.0) + (crevice_wall_size.x / 2.0) * angle_rad.cos();
    let vert_offset = (crevice_wall_size.x / 2.0) * angle_rad.sin();
    let wall_center_y = BASE_Y + crevice_wall_size.y / 2.0;

    let wall_material = create_material_with_uv(
        TEX_WALL,
        crevice_wall_size,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );

    // Wall 1 (Left)
    let transform1 = Transform::from_xyz(
        section_center_x - horz_offset,
        wall_center_y - vert_offset,
        0.0,
    )
    .with_rotation(Quat::from_rotation_z(angle_rad));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(crevice_wall_size))),
        MeshMaterial3d(wall_material.clone()),
        transform1,
        RigidBody::Static,
        Collider::cuboid(
            crevice_wall_size.x / 2.0,
            crevice_wall_size.y / 2.0,
            crevice_wall_size.z / 2.0,
        ),
        Geometry,
        Name::new("CreviceWall_Left"), // VisibilityBundle::default(),
    ));

    // Wall 2 (Right)
    let transform2 = Transform::from_xyz(
        section_center_x + horz_offset,
        wall_center_y - vert_offset,
        0.0,
    )
    .with_rotation(Quat::from_rotation_z(-angle_rad));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(crevice_wall_size))),
        MeshMaterial3d(wall_material),
        transform2,
        RigidBody::Static,
        Collider::cuboid(
            crevice_wall_size.x / 2.0,
            crevice_wall_size.y / 2.0,
            crevice_wall_size.z / 2.0,
        ),
        Geometry,
        Name::new("CreviceWall_Right"), // VisibilityBundle::default(),
    ));
    current_x_offset += OBJECT_SPACING;

    // --- Moving Platforms ---
    // (Keep animation setup the same, just change the spawn pattern)

    let platform_size = Vec3::new(
        MOVING_PLATFORM_WIDTH * MAP_SCALER,
        MOVING_PLATFORM_HEIGHT * MAP_SCALER,
        MOVING_PLATFORM_DEPTH * MAP_SCALER,
    );
    let platform_material = create_material_with_uv(
        TEX_PLATFORM,
        platform_size,
        UV_TILE_FACTOR,
        &level_assets,
        &mut materials,
        &fallback_material_handle,
    );

    // 1. Vertical Moving Platform
    let section_center_x = current_x_offset;
    let platform_start_pos = Vec3::new(section_center_x, BASE_Y + platform_size.y, 0.0);
    let platform_end_pos = platform_start_pos + Vec3::Y * PLATFORM_VERTICAL_DISTANCE * MAP_SCALER;
    let platform_name = Name::new("Platform_Vertical");
    let target_id = AnimationTargetId::from_name(&platform_name);

    // --- Animation Setup (Inlined) ---
    let mut clip = AnimationClip::default();
    let translation_curve = EasingCurve::new(
        // Explicitly creates EasingCurve<Vec3, f32> or similar
        platform_start_pos,
        platform_end_pos,
        EaseFunction::SineInOut,
    )
    .reparametrize_linear(Interval::new(0.0, PLATFORM_ANIMATION_DURATION).unwrap())
    .expect("Curve creation failed")
    .ping_pong()
    .expect("Ping pong failed");

    // Create the AnimatableCurve with concrete types
    let animatable_translation_curve = AnimatableCurve::new(
        animated_field!(Transform::translation), // Property type is known
        translation_curve,                       // Curve type is known (EasingCurve<Vec3, ...>)
    );
    clip.add_curve_to_target(target_id, animatable_translation_curve);

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    // --- End Animation Setup ---

    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();

    let platform_entity = commands
        .spawn((
            // Components:
            Mesh3d(meshes.add(Cuboid::from_size(platform_size))),
            MeshMaterial3d(platform_material.clone()),
            Transform::from_translation(platform_start_pos),
            // Physics: (Using KinematicPositionBased is generally recommended for smooth KCC interaction)
            RigidBody::Kinematic, // Changed from ::Kinematic
            Collider::cuboid(
                platform_size.x / 2.0,
                platform_size.y / 2.0,
                platform_size.z / 2.0,
            ),
            // Animation:
            platform_name, // Name used for target_id
            AnimationGraphHandle(graph_handle),
            player,
            // Other:
            Geometry,
            // VisibilityBundle::default(),
        ))
        .id();
    commands.entity(platform_entity).insert(AnimationTarget {
        id: target_id,
        player: platform_entity,
    });
    current_x_offset += OBJECT_SPACING;

    // 2. Horizontal Moving Platform
    let section_center_x = current_x_offset;
    let platform_start_pos = Vec3::new(
        section_center_x - PLATFORM_HORIZONTAL_DISTANCE * MAP_SCALER / 2.0,
        BASE_Y + platform_size.y,
        -5.0 * MAP_SCALER,
    );
    let platform_end_pos = platform_start_pos + Vec3::X * PLATFORM_HORIZONTAL_DISTANCE * MAP_SCALER;
    let platform_name = Name::new("Platform_Horizontal");
    let target_id = AnimationTargetId::from_name(&platform_name);

    // --- Animation Setup (Inlined) ---
    let mut clip = AnimationClip::default();
    let translation_curve = EasingCurve::new(
        // Explicit type
        platform_start_pos,
        platform_end_pos,
        EaseFunction::SineInOut,
    )
    .reparametrize_linear(Interval::new(0.0, PLATFORM_ANIMATION_DURATION).unwrap())
    .expect("Curve creation failed")
    .ping_pong()
    .expect("Ping pong failed");

    // Create the AnimatableCurve with concrete types
    let animatable_translation_curve =
        AnimatableCurve::new(animated_field!(Transform::translation), translation_curve);
    clip.add_curve_to_target(target_id, animatable_translation_curve);

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    // --- End Animation Setup ---

    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();

    let platform_entity = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(platform_size))),
            MeshMaterial3d(platform_material.clone()),
            Transform::from_translation(platform_start_pos),
            RigidBody::Kinematic, // Changed from ::Kinematic
            Collider::cuboid(
                platform_size.x / 2.0,
                platform_size.y / 2.0,
                platform_size.z / 2.0,
            ),
            platform_name,
            AnimationGraphHandle(graph_handle),
            player,
            Geometry, // VisibilityBundle::default(),
        ))
        .id();
    commands.entity(platform_entity).insert(AnimationTarget {
        id: target_id,
        player: platform_entity,
    });
    current_x_offset += OBJECT_SPACING;

    // 3. Rotating Platform
    let section_center_x = current_x_offset;
    let platform_pos = Vec3::new(section_center_x, BASE_Y + platform_size.y, 0.0);
    let platform_name = Name::new("Platform_Rotating");
    let target_id = AnimationTargetId::from_name(&platform_name);

    // --- Animation Setup (Inlined) ---
    let mut clip = AnimationClip::default();
    let rotation_curve = EasingCurve::new(
        // Explicit type EasingCurve<Quat, f32> or similar
        Quat::IDENTITY,
        Quat::from_rotation_y(PLATFORM_ROTATION_ANGLE),
        EaseFunction::Linear,
    )
    .reparametrize_linear(Interval::new(0.0, PLATFORM_ANIMATION_DURATION * 2.0).unwrap()) // Adjust duration
    .expect("Curve creation failed")
    .repeat(0); // Use repeat() for continuous rotation without explicit count

    // Create the AnimatableCurve with concrete types
    let animatable_rotation_curve = AnimatableCurve::new(
        animated_field!(Transform::rotation), // Property type known
        rotation_curve.unwrap(),              // Curve type is known (EasingCurve<Quat, ...>)
    );
    clip.add_curve_to_target(target_id, animatable_rotation_curve);

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    // --- End Animation Setup ---

    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();

    let platform_entity = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(platform_size))),
            MeshMaterial3d(platform_material), // Last use of this handle
            Transform::from_translation(platform_pos),
            RigidBody::Kinematic, // Changed from ::Kinematic
            Collider::cuboid(
                platform_size.x / 2.0,
                platform_size.y / 2.0,
                platform_size.z / 2.0,
            ),
            platform_name,
            AnimationGraphHandle(graph_handle),
            player,
            Geometry, // VisibilityBundle::default(),
        ))
        .id();
    commands.entity(platform_entity).insert(AnimationTarget {
        id: target_id,
        player: platform_entity,
    });

    // current_x_offset += OBJECT_SPACING;

    info!("Level creation complete using component spawning and UV transforms.");
}
