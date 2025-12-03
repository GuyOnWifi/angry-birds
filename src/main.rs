use avian2d::{math::Vector, prelude::*};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(Gravity(Vec2::NEG_Y * 9.8 * 100.0)) // Scale gravity for pixels
        .init_resource::<DragState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (input_system, time_control_system))
        .run();
}

#[derive(Component)]
struct Bird;

#[derive(Component)]
struct Pig;

#[derive(Component)]
struct Block;

#[derive(Component)]
struct Slingshot;

#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
    start_pos: Vec2,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut time: ResMut<Time<Physics>>) {
    // Pause time to view structure
    // time.pause();

    // Camera
    commands.spawn(Camera2d);

    // Background
    commands.spawn((
        Sprite::from_image(asset_server.load("background.png")),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    // Ground
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.8, 0.2), Vec2::new(1000.0, 50.0)),
        Transform::from_xyz(0.0, -300.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(1000.0, 50.0),
    ));

    // Slingshot
    let slingshot_pos = Vec2::new(-300.0, -220.0);

    // Right part (Back)
    commands.spawn((
        Sprite::from_image(asset_server.load("slingshot_right.png")),
        Transform::from_xyz(slingshot_pos.x + 20.0, slingshot_pos.y + 30.0, 1.0),
        Slingshot,
    ));

    // Left part (Front)
    commands.spawn((
        Sprite::from_image(asset_server.load("slingshot_left.png")),
        Transform::from_xyz(slingshot_pos.x - 5.0, slingshot_pos.y + 80.0, 3.0), // Higher Z to be in front of bird
        Slingshot,
    ));

    // Bird (Ready to launch)
    commands.spawn((
        Sprite::from_image(asset_server.load("pigs/pig_silly.png")),
        Transform::from_xyz(slingshot_pos.x, slingshot_pos.y + 100.0, 2.0),
        RigidBody::Kinematic, // Kinematic while waiting
        Collider::circle(23.0),
        Bird,
    ));

    // Complex Tower
    spawn_game(&mut commands, &asset_server);
}

#[derive(Clone, Copy)]
enum BlockMaterial {
    Wood,
    Steel,
}

#[derive(Clone, Copy)]
enum BlockShape {
    SquareLarge,
    SquareMedium,
    SquareSmall,
    LongBeam,
    ShortBeam,
    Triangle,
}

struct BlockCreator {
    material: BlockMaterial,
    shape: BlockShape,
    pos: Vec2,
    rotation: Quat,
}

struct PigCreator {
    pos: Vec2,
    pig_type: PigType,
}

fn get_game_layout() -> (Vec<BlockCreator>, Vec<PigCreator>) {
    let mut blocks = Vec::new();
    let mut pigs = Vec::new();

    let center_x = 300.0;
    let ground_top = -275.0;

    // --- The "Thanklessly Maintaining" Base ---
    // Right: Single vertical stone block holding up the right side (The unstable part)
    // Using ShortBeam (83x41) rotated 90 degrees -> 41 wide, 83 tall
    let right_support_h = 83.0;
    let right_support_pos = Vec2::new(center_x + 120.0, ground_top + right_support_h / 2.0);
    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::ShortBeam,
        pos: right_support_pos,
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });

    // Left: A solid base of stone
    // Using SquareLarge (82x82)
    let left_support_h = 82.0;
    let left_support_pos = Vec2::new(center_x - 120.0, ground_top + left_support_h / 2.0);
    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::SquareLarge,
        pos: left_support_pos,
        rotation: Quat::IDENTITY,
    });

    // Pig under the floor
    pigs.push(PigCreator {
        pos: Vec2::new(center_x, ground_top + 23.0),
        pig_type: PigType::Normal,
    });

    // --- The Main Floor Plank ---
    // Spanning across the two supports.
    // We need a very long span. Let's use two LongBeams (167x20) overlapping or end-to-end.
    // Supports are at -120 and +120 (dist 240). LongBeam is 167.
    // Let's put one centered, but it might fall.
    // Let's put two LongBeams side by side to make a wide platform.
    let floor_y = ground_top + right_support_h + 10.0; // Sitting on top of the 83.0 high support

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(center_x - 80.0, floor_y),
        rotation: Quat::IDENTITY,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(center_x + 80.0, floor_y),
        rotation: Quat::IDENTITY,
    });

    // --- Left Tower (The "Box") ---
    // Sitting on the left side of the floor.
    let left_tower_x = center_x - 120.0;
    let mut current_y = floor_y + 10.0; // Top of floor

    // Walls: Vertical LongBeams (167 tall)
    let wall_h = 167.0;
    let wall_y = current_y + wall_h / 2.0;

    // Left Wall
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(left_tower_x - 70.0, wall_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });
    // Right Wall (Shared with middle?)
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(left_tower_x + 70.0, wall_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });

    // Shelves inside the box (The "Glass" windows replaced by Wood)
    // Lower Shelf
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(left_tower_x, current_y + 40.0),
        rotation: Quat::IDENTITY,
    });
    // Pig on lower shelf
    pigs.push(PigCreator {
        pos: Vec2::new(left_tower_x, current_y + 40.0 + 20.0 + 23.0),
        pig_type: PigType::Normal,
    });

    // Upper Shelf
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(left_tower_x, current_y + 100.0),
        rotation: Quat::IDENTITY,
    });
    // Pig on upper shelf
    pigs.push(PigCreator {
        pos: Vec2::new(left_tower_x, current_y + 100.0 + 20.0 + 23.0),
        pig_type: PigType::Normal,
    });

    // Ceiling of Left Tower
    let ceiling_y = current_y + wall_h + 10.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Steel, // Stone slab on top
        shape: BlockShape::LongBeam,
        pos: Vec2::new(left_tower_x, ceiling_y),
        rotation: Quat::IDENTITY,
    });

    // Debris on top of Left Tower
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(left_tower_x - 40.0, ceiling_y + 20.0 + 41.5),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(left_tower_x + 20.0, ceiling_y + 20.0 + 10.0),
        rotation: Quat::IDENTITY,
    });

    // --- Right Tower (The Tall Unstable One) ---
    let right_tower_x = center_x + 80.0;
    current_y = floor_y + 10.0;

    // Level 1: Vertical Wood Beams (Short or Long? Image looks like stacked frames)
    // Let's use Vertical ShortBeams (83 tall) for a more segmented look
    let l1_h = 83.0;
    let l1_y = current_y + l1_h / 2.0;

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x - 50.0, l1_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x + 50.0, l1_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });

    // Pig in Level 1
    pigs.push(PigCreator {
        pos: Vec2::new(right_tower_x, current_y + 23.0),
        pig_type: PigType::Normal,
    });

    // Level 1 Ceiling
    current_y += l1_h + 10.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(right_tower_x, current_y),
        rotation: Quat::IDENTITY,
    });

    // Level 2: More Vertical Beams (The "Glass" part replaced by Wood)
    // Using LongBeams vertical here for height
    let l2_h = 167.0;
    let l2_y = current_y + 10.0 + l2_h / 2.0;

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(right_tower_x - 40.0, l2_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(right_tower_x + 40.0, l2_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });

    // Pig in Level 2
    pigs.push(PigCreator {
        pos: Vec2::new(right_tower_x, current_y + 10.0 + 23.0),
        pig_type: PigType::Normal,
    });

    // Level 2 Ceiling
    current_y += 10.0 + l2_h + 10.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(right_tower_x, current_y),
        rotation: Quat::IDENTITY,
    });

    // Level 3: The "Penthouse"
    // Short vertical beams
    let l3_h = 83.0;
    let l3_y = current_y + 10.0 + l3_h / 2.0;

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x - 30.0, l3_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x + 30.0, l3_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    });

    // King Pig at the top
    pigs.push(PigCreator {
        pos: Vec2::new(right_tower_x, current_y + 10.0 + 23.0),
        pig_type: PigType::King,
    });

    // Level 3 Ceiling (Stone)
    current_y += 10.0 + l3_h + 10.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x, current_y),
        rotation: Quat::IDENTITY,
    });

    // Top Crown
    // A couple of small blocks balancing
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(right_tower_x - 20.0, current_y + 10.0 + 10.0),
        rotation: Quat::IDENTITY,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(right_tower_x + 20.0, current_y + 10.0 + 10.0),
        rotation: Quat::IDENTITY,
    });

    (blocks, pigs)
}

fn spawn_game(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let (blocks, pigs) = get_game_layout();
    for block in blocks {
        spawn_block(
            commands,
            asset_server,
            block.material,
            block.shape,
            block.pos,
            block.rotation,
        );
    }

    for pig in pigs {
        spawn_pig(commands, asset_server, pig.pig_type, pig.pos);
    }
}

enum PigType {
    King,
    Normal,
}

fn spawn_block(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    material: BlockMaterial,
    shape: BlockShape,
    pos: Vec2,
    rotation: Quat,
) {
    let material_str = match material {
        BlockMaterial::Steel => "steel/steel",
        BlockMaterial::Wood => "wood/wood",
    };

    let shape_str = match shape {
        BlockShape::SquareLarge => "square_large",
        BlockShape::SquareMedium => "square_medium",
        BlockShape::SquareSmall => "square_small",
        BlockShape::LongBeam => "beam_long",
        BlockShape::ShortBeam => "beam_short",
        BlockShape::Triangle => "triangle",
    };

    let collider = match shape {
        BlockShape::SquareLarge => Collider::rectangle(82.0, 82.0),
        BlockShape::SquareMedium => Collider::rectangle(41.0, 41.0),
        BlockShape::SquareSmall => Collider::rectangle(20.0, 20.0),
        BlockShape::LongBeam => Collider::rectangle(167.0, 20.0),
        BlockShape::ShortBeam => Collider::rectangle(83.0, 41.0),
        BlockShape::Triangle => Collider::triangle(
            Vector::new(0.0, 41.0),
            Vector::new(-41.0, -41.0),
            Vector::new(41.0, -41.0),
        ),
    };

    let asset_path = format!("blocks/{}_{}.png", material_str, shape_str);
    commands.spawn((
        Sprite::from_image(asset_server.load(asset_path)),
        Transform::from_xyz(pos.x, pos.y, 0.0).with_rotation(rotation),
        RigidBody::Dynamic,
        collider,
    ));
}

fn spawn_pig(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    pig_type: PigType,
    pos: Vec2,
) {
    let (path, radius) = match pig_type {
        PigType::King => ("pigs/pig_king.png", 70.0),
        PigType::Normal => ("pigs/pig_normal.png", 23.0),
    };

    commands.spawn((
        Sprite::from_image(asset_server.load(path)),
        Transform::from_xyz(pos.x, pos.y, 0.0),
        RigidBody::Dynamic,
        Collider::circle(radius),
        Pig,
    ));
}

fn input_system(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut bird_q: Query<(Entity, &mut Transform), With<Bird>>,
) {
    let Some((camera, camera_transform)) = camera_q.iter().next() else {
        return;
    };
    let Some(window) = windows.iter().next() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            if mouse_button.just_pressed(MouseButton::Left) {
                // Check if clicking near bird (simplified)
                if let Some((_, transform)) = bird_q.iter().next() {
                    if transform.translation.truncate().distance(world_pos) < 50.0 {
                        drag_state.is_dragging = true;
                        drag_state.start_pos = world_pos;
                    }
                }
            }

            if drag_state.is_dragging {
                if mouse_button.pressed(MouseButton::Left) {
                    // Drag bird
                    if let Some((_, mut transform)) = bird_q.iter_mut().next() {
                        transform.translation.x = world_pos.x;
                        transform.translation.y = world_pos.y;
                    }
                } else {
                    // Release
                    drag_state.is_dragging = false;
                    if let Some((entity, _)) = bird_q.iter().next() {
                        commands.entity(entity).insert(RigidBody::Dynamic);
                        let force = (drag_state.start_pos - world_pos) * 15.0; // Launch force
                        commands.entity(entity).insert(LinearVelocity(force));
                    }
                }
            }
        }
    }
}

fn time_control_system(keyboard: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Physics>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }

    if time.is_paused() {
        if keyboard.just_pressed(KeyCode::KeyS) {
            // Step by fixed timestep (usually 1/60)
            time.advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
        }
    }
}
