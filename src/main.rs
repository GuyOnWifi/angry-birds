use avian2d::{math::Vector, prelude::*};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::seq::IndexedRandom;

const PIG_TEXT: &[&str] = &[
    "The XZ utils incident: where a hacker snuck a virus into burnt-out maintainer's code",
    "The FFMPEG incident: where Microsoft demanded volunteers to fix their \'high priority\' issue",
    "Amazon vs Redis: When Amazon wrapped Redis' code and made billions by selling it as a cloud service",
    "OpenSSL: Funding cuts for a library used by most internet encryption",
];

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }),
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin::default(),
        ))
        .add_plugins(EguiPlugin::default())
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(Gravity(Vec2::NEG_Y * 9.8 * 100.0)) // Scale gravity for pixels
        .insert_resource(SlingshotState {
            desc: "Launch a pig to find out what disaster you are about to unleash!\n\nThe XZ utils incident: where a hacker snuck a virus into burnt-out maintainer's code".into(),
            ..Default::default()
        })
        .insert_resource(RespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input_system,
                time_control_system,
                block_destruction_system,
                respawn_bird_system,
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                pig_info_system,
                hover_info_system,
                restart_ui_system,
                pig_destruction_system,
            ),
        )
        .run();
}

#[derive(Component)]
struct Bird;

#[derive(Component)]
struct OnSlingshot;

#[derive(Component)]
struct Pig;

#[derive(Component)]
struct Block;

#[derive(Component)]
struct Invisible;

#[derive(Component)]
struct Slingshot;

#[derive(Component)]
struct BlockDescription(String);

#[derive(Resource, Default)]
struct SlingshotState {
    is_dragging: bool,
    start_pos: Vec2,
    desc: String,
}

#[derive(Resource)]
struct RespawnTimer(Timer);

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
    spawn_bird(&mut commands, &asset_server);

    // Complex Tower
    spawn_game(&mut commands, &asset_server);
}

#[derive(Component, Clone, Copy)]
enum BlockMaterial {
    Wood,
    Steel,
    Invisible,
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
    description: Option<String>,
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
        description: Some("FFMPEG: A video decoding library that powers Spotify, Instagram, Youtube, Tiktok and more.".into()),
    });

    // Left: A solid base of stone
    // Using SquareLarge (82x82)
    let middle_support_h = 83.0;
    let left_support_pos = Vec2::new(center_x, ground_top + middle_support_h / 2.0);
    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::ShortBeam,
        pos: left_support_pos,
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: Some("OpenSSL: Internet traffic encryption for secure communication. Powers banking and e-commerce".into()),
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
        description: Some("The Linux Kernel. The biggest open source project, with over 40 million lines of code. Powers virtually every server hosting Internet content.".into()),
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
        description: None,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(center_x + 80.0, floor_y),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // Left Wing
    // These blocks are placed relative to the right tower's x-coordinate,
    // but at the same y-level as the main floor planks.
    // Assuming 'right_tower_x' is intended to be 'center_x' for these wings,
    // or that these are meant to be part of the main floor structure.
    // Given the instruction "Add descriptions to the wing blocks", and the provided
    // code snippet, it seems these are new blocks.
    // The original code does not define `l4_y` at this point, so we'll use `floor_y`.
    // The `right_tower_x` variable is defined later, so we'll use `center_x` for now,
    // or assume these are meant to be placed relative to the main structure.
    // For faithfulness to the instruction, we'll use `center_x` for placement
    // and `floor_y` for height, as `l4_y` is not defined here.

    // Even more unstable layer
    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(center_x - 50.0, floor_y + 10.0 + 83.0 / 2.0),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: Some("PyTorch: The open-source AI machine learning tool that powers all AI training, including ChatGPT".into()),
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(center_x + 50.0, floor_y + 10.0 + 83.0 / 2.0),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: Some("LLVM & GCC: Open source tools that run code. Every programmer, every programming language likely has had some amount of LLVM or GCC in it.".into()),
    });

    // invisible supports
    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(center_x, floor_y + 10.0 + 83.0 / 2.0),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(center_x + 120.0, floor_y + 10.0 + 83.0 / 2.0),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(center_x - 120.0, floor_y + 10.0 + 83.0 / 2.0),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });

    // floor of the unstable layer (slight gap is to allow a metal box to "pin down")
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(center_x - 80.0 - 7.0, floor_y + 83.0 + 10.0 + 10.0),
        rotation: Quat::IDENTITY,
        description: None,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(center_x + 80.0 + 5.0, floor_y + 83.0 + 10.0 + 10.0),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // --- Left Tower (The "Box") ---
    // Sitting on the left side of the floor.
    let left_tower_x = center_x - 90.0;
    let mut current_y = floor_y + 10.0 + 83.0 + 20.0; // Top of floor

    // Walls: Vertical LongBeams (167 tall)
    let wall_h = 167.0;
    let wall_y = current_y + wall_h / 2.0;

    // Left Wall
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(left_tower_x - 70.0, wall_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });
    // Right Wall (Shared with middle?)
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(left_tower_x + 70.0, wall_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });

    // Shelves inside the box (The "Glass" windows replaced by Wood)
    // Lower Shelf
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(left_tower_x, current_y + 40.0),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // Upper Shelf
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(left_tower_x, current_y + 100.0),
        rotation: Quat::IDENTITY,
        description: None,
    });
    // Pig on upper shelf
    pigs.push(PigCreator {
        pos: Vec2::new(left_tower_x, current_y + 100.0 + 20.0 + 23.0),
        pig_type: PigType::BombBird,
    });

    // Ceiling of Left Tower
    let ceiling_y = current_y + wall_h + 10.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood, // Stone slab on top
        shape: BlockShape::LongBeam,
        pos: Vec2::new(left_tower_x, ceiling_y),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // Left Tower "jutting out"

    blocks.push(BlockCreator {
        material: BlockMaterial::Steel, // Stone slab on top
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(left_tower_x - 50.0, ceiling_y + 10.0 + 83.0 / 2.0),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: Some("Git: The \'Google Docs\' of coding".into()),
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood, // Stone slab on top
        shape: BlockShape::LongBeam,
        pos: Vec2::new(left_tower_x - 50.0, ceiling_y + 10.0 + 10.0 + 83.0),
        rotation: Quat::IDENTITY,
        description: None,
    });

    let jutted_floor_x = left_tower_x - 50.0;
    let jutted_floor_y = ceiling_y + 10.0 + 10.0 + 83.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(
            jutted_floor_x - 163.0 / 2.0 + 10.0,
            jutted_floor_y + 81.0 / 2.0 + 10.0,
        ),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: Some("Redis: The ultra-fast data cache for high-performance websites".into()),
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(jutted_floor_x - 163.0 / 2.0 + 10.0, jutted_floor_y - 20.0),
        rotation: Quat::IDENTITY,
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::SquareLarge,
        pos: Vec2::new(
            jutted_floor_x + 163.0 / 2.0 - 10.0,
            jutted_floor_y + 81.0 / 2.0 + 10.0,
        ),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(
            jutted_floor_x + 163.0 / 2.0 - 10.0 - 163.0,
            jutted_floor_y + 81.0 + 10.0 + 10.0,
        ),
        rotation: Quat::IDENTITY,
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(jutted_floor_x - 163.0 - 10.0, jutted_floor_y + 81.0),
        rotation: Quat::IDENTITY,
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(
            jutted_floor_x + 163.0 / 2.0 - 10.0 - 326.0,
            jutted_floor_y + 81.0 + 10.0 + 10.0,
        ),
        rotation: Quat::IDENTITY,
        description: None,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(jutted_floor_x - 163.0 - 163.0, jutted_floor_y + 81.0),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // King Pig at the top
    pigs.push(PigCreator {
        pos: Vec2::new(jutted_floor_x - 81.0, jutted_floor_y + 81.0 + 90.0 + 10.0),
        pig_type: PigType::King,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(
            jutted_floor_x - 81.0 + 81.0,
            jutted_floor_y + 81.0 + 90.0 + 10.0,
        ),
        rotation: Quat::IDENTITY,
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(
            jutted_floor_x - 81.0 - 81.0,
            jutted_floor_y + 81.0 + 90.0 + 10.0,
        ),

        rotation: Quat::IDENTITY,
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Steel,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(
            jutted_floor_x + 163.0 / 2.0 - 10.0 - 326.0 - 66.0,
            jutted_floor_y + 81.0 + 10.0 + 10.0 + 41.0 + 10.0,
        ),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: Some(
            "PostgreSQL: The database that powers all of our data storage and retrieval".into(),
        ),
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(
            jutted_floor_x + 163.0 / 2.0 - 10.0 - 326.0 - 66.0,
            jutted_floor_y + 81.0 + 10.0 + 10.0 + 41.0 + 10.0 + 41.0,
        ),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // King Pig at the top
    pigs.push(PigCreator {
        pos: Vec2::new(
            jutted_floor_x + 163.0 / 2.0 - 10.0 - 326.0 - 66.0,
            jutted_floor_y + 81.0 + 10.0 + 10.0 + 41.0 + 10.0 + 41.0 + 85.0,
        ),
        pig_type: PigType::King,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(
            jutted_floor_x + 163.0 / 2.0 - 10.0 - 326.0 - 66.0 - 86.0,
            jutted_floor_y + 81.0 + 10.0 + 10.0 + 41.0 + 10.0 + 41.0 + 85.0,
        ),
        rotation: Quat::IDENTITY,
        description: None,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(
            jutted_floor_x + 163.0 / 2.0 - 10.0 - 326.0 - 66.0 + 86.0,
            jutted_floor_y + 81.0 + 10.0 + 10.0 + 41.0 + 10.0 + 41.0 + 85.0,
        ),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // --- Right Tower (The Tall Unstable One) ---
    let right_tower_x = center_x + 100.0;
    current_y = floor_y + 10.0 + 83.0 + 20.0;

    // Level 1: Vertical Wood Beams (Short or Long? Image looks like stacked frames)
    // Let's use Vertical ShortBeams (83 tall) for a more segmented look
    let l1_h = 83.0;
    let l1_y = current_y + l1_h / 2.0;

    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x - 50.0, l1_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x + 50.0, l1_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });

    // Pig in Level 1
    pigs.push(PigCreator {
        pos: Vec2::new(right_tower_x, current_y + 23.0),
        pig_type: PigType::TriangleBird,
    });

    // Level 1 Ceiling
    current_y += l1_h + 10.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(right_tower_x, current_y),
        rotation: Quat::IDENTITY,
        description: None,
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
        description: None,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(right_tower_x + 40.0, l2_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });

    // Pig in Level 2
    pigs.push(PigCreator {
        pos: Vec2::new(right_tower_x, current_y + 10.0 + 23.0),
        pig_type: PigType::BlueBird,
    });

    // Level 2 Ceiling
    current_y += 10.0 + l2_h + 10.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::LongBeam,
        pos: Vec2::new(right_tower_x, current_y),
        rotation: Quat::IDENTITY,
        description: None,
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
        description: None,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x + 30.0, l3_y),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        description: None,
    });

    // King Pig at the top
    pigs.push(PigCreator {
        pos: Vec2::new(right_tower_x, l3_y + 10.0 + 23.0 + 70.0 + 50.0),
        pig_type: PigType::King,
    });

    blocks.push(BlockCreator {
        material: BlockMaterial::Invisible,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(right_tower_x + 84.0, l3_y + 10.0 + 23.0 + 70.0 + 50.0),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // Level 3 Ceiling (Stone)
    current_y += 10.0 + l3_h + 10.0;
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::ShortBeam,
        pos: Vec2::new(right_tower_x, current_y),
        rotation: Quat::IDENTITY,
        description: None,
    });

    // Top Crown
    // A couple of small blocks balancing
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(right_tower_x - 25.0, current_y + 10.0 + 10.0 + 5.0),
        rotation: Quat::IDENTITY,
        description: None,
    });
    blocks.push(BlockCreator {
        material: BlockMaterial::Wood,
        shape: BlockShape::SquareSmall,
        pos: Vec2::new(right_tower_x + 25.0, current_y + 10.0 + 10.0 + 5.0),
        rotation: Quat::IDENTITY,
        description: None,
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
            block.description,
        );
    }

    for pig in pigs {
        spawn_pig(commands, asset_server, pig.pig_type, pig.pos);
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
enum PigType {
    King,
    Normal,
    RedBird,
    BombBird,
    TriangleBird,
    EggBird,
    BlueBird,
}

fn spawn_block(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    material: BlockMaterial,
    shape: BlockShape,
    pos: Vec2,
    rotation: Quat,
    description: Option<String>,
) {
    let material_str = match material {
        BlockMaterial::Steel => "steel/steel",
        BlockMaterial::Wood => "wood/wood",
        BlockMaterial::Invisible => "",
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

    let asset_path = if matches!(material, BlockMaterial::Invisible) {
        "".to_string()
    } else {
        format!("blocks/{}_{}.png", material_str, shape_str)
    };
    let mut cmd = commands.spawn((
        Sprite::from_image(asset_server.load(asset_path)),
        Transform::from_xyz(pos.x, pos.y, 0.0).with_rotation(rotation),
        if matches!(material, BlockMaterial::Invisible) {
            RigidBody::Static
        } else {
            RigidBody::Dynamic
        },
        collider,
        Block,
    ));

    if matches!(material, BlockMaterial::Invisible) {
        cmd.insert(Invisible);
    }
    cmd.insert(material);

    if let Some(desc) = description {
        cmd.insert(BlockDescription(desc));
    }
}

fn spawn_pig(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    pig_type: PigType,
    pos: Vec2,
) {
    let (path, collider) = match pig_type {
        PigType::King => ("pigs/pig_king.png", Collider::circle(70.0)),
        PigType::Normal => ("pigs/pig_normal.png", Collider::circle(23.0)),
        PigType::RedBird => ("birds/red.png", Collider::circle(22.0)),
        PigType::BombBird => ("birds/black.png", Collider::circle(42.0)),
        PigType::TriangleBird => (
            "birds/yellow.png",
            Collider::triangle(
                Vector::new(0.0, 39.0),
                Vector::new(-39.0, -39.0),
                Vector::new(39.0, -39.0),
            ),
        ),
        PigType::EggBird => ("birds/white.png", Collider::capsule(40.0, 60.0)), // Approximate capsule
        PigType::BlueBird => ("birds/blue.png", Collider::circle(22.0)),
    };

    commands.spawn((
        Sprite::from_image(asset_server.load(path)),
        Transform::from_xyz(pos.x, pos.y, 0.0),
        RigidBody::Dynamic,
        collider,
        CollidingEntities::default(),
        Pig,
        pig_type,
    ));
}

fn spawn_bird(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let slingshot_pos = Vec2::new(-300.0, -220.0);

    commands.spawn((
        Sprite::from_image(asset_server.load("pigs/pig_silly.png")),
        Transform::from_xyz(slingshot_pos.x, slingshot_pos.y + 100.0, 2.0),
        RigidBody::Kinematic, // Kinematic while waiting
        Collider::circle(22.0),
        CollidingEntities::default(),
        SweptCcd::default(),
        ColliderDensity(5.0),
        Bird,
        OnSlingshot,
    ));
}

fn respawn_bird_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<RespawnTimer>,
    bird_q: Query<Entity, With<OnSlingshot>>,
    mut slingshot_state: ResMut<SlingshotState>,
) {
    if bird_q.iter().next().is_some() {
        // defined a bird, so reset timer
        timer.0.reset();
        return;
    }

    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        spawn_bird(&mut commands, &asset_server);
        // Update text
        let mut rng = rand::rng();
        if let Some(text) = PIG_TEXT.choose(&mut rng) {
            slingshot_state.desc = text.to_string();
        }
    }
}

fn input_system(
    mut commands: Commands,
    mut drag_state: ResMut<SlingshotState>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut bird_q: Query<(Entity, &mut Transform), (With<Bird>, With<OnSlingshot>)>,
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
                        commands.entity(entity).remove::<OnSlingshot>();
                    }
                }
            }
        }
    }
}

fn pig_destruction_system(
    mut commands: Commands,
    pig_q: Query<(Entity, &LinearVelocity, &CollidingEntities), With<Pig>>,
) {
    for (entity, velocity, colliding_entities) in pig_q.iter() {
        if !colliding_entities.is_empty() && velocity.length() > 600.0 {
            commands.entity(entity).despawn();
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

fn pig_info_system(mut contexts: EguiContexts, slingshot: Res<SlingshotState>) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Pig Info: What Disaster Will You Launch This Time?")
            .default_pos((0.0, 200.0))
            .show(ctx, |ui| ui.label(slingshot.desc.clone()));
    }
}

fn restart_ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    query: Query<Entity, Or<(With<Block>, With<Pig>, With<Bird>)>>,
    mut slingshot_state: ResMut<SlingshotState>,
    mut respawn_timer: ResMut<RespawnTimer>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Game Control")
            .default_pos((0.0, 10.0))
            .show(ctx, |ui| {
                if ui.button("Restart Level").clicked() {
                    // Despawn all game entities
                    for entity in query.iter() {
                        commands.entity(entity).despawn();
                    }

                    // Reset state
                    slingshot_state.desc = "Restarted!".into();
                    respawn_timer.0.reset();

                    // Respawn level
                    spawn_game(&mut commands, &asset_server);
                    spawn_bird(&mut commands, &asset_server);
                }
            });
    }
}

fn hover_info_system(
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    block_desc_q: Query<&BlockDescription>,
) {
    let Some((camera, camera_transform)) = camera_q.iter().next() else {
        return;
    };
    let Some(window) = windows.iter().next() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            // Raycast or point projection? Point projection is easier for "hovering".
            // Let's check for entities at the cursor position.
            // We'll use a small radius for "picking".
            let intersections =
                spatial_query.point_intersections(world_pos, &SpatialQueryFilter::default());

            let desc_ui = egui::Window::new("Block Info: What's Holding up the Internet?")
                .default_pos((0.0, 100.0));

            let mut content_message =
                "Hover over a metal block for more info (they're also harder to destroy)!";

            for entity in intersections {
                if let Ok(desc) = block_desc_q.get(entity) {
                    println!("desc: {}", desc.0);
                    content_message = &desc.0;
                    break; // Only show one
                }
            }

            desc_ui.show(contexts.ctx_mut().unwrap(), |ui| {
                ui.label(content_message);
            });
        }
    }
}

fn block_destruction_system(
    mut commands: Commands,
    block_q: Query<&BlockMaterial, With<Block>>,
    invisible_q: Query<Entity, With<Invisible>>,
    bird_q: Query<(&LinearVelocity, &CollidingEntities), With<Bird>>,
) {
    let mut any_destroyed = false;
    for (velocity, colliding_entities) in bird_q.iter() {
        let mag = velocity.length();
        for &hit_entity in colliding_entities.iter() {
            if let Ok(material) = block_q.get(hit_entity) {
                let threshold = match material {
                    BlockMaterial::Steel => 800.0,
                    _ => 600.0,
                };

                if mag > threshold {
                    commands.entity(hit_entity).despawn();
                    any_destroyed = true;
                }
            }
        }
    }

    if any_destroyed {
        for entity in invisible_q.iter() {
            commands.entity(entity).despawn();
        }
    }
}
