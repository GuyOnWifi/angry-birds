use avian2d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }),
            PhysicsPlugins::default(),
        ))
        .insert_resource(Gravity(Vec2::NEG_Y * 9.8 * 100.0)) // Scale gravity for pixels
        .init_resource::<DragState>()
        .add_systems(Startup, setup)
        .add_systems(Update, input_system)
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    commands.spawn((
        Sprite::from_image(asset_server.load("slingshot.png")),
        Transform::from_xyz(slingshot_pos.x, slingshot_pos.y, 1.0),
        Slingshot,
    ));

    // Bird (Ready to launch)
    commands.spawn((
        Sprite::from_image(asset_server.load("bird_red.png")),
        Transform::from_xyz(slingshot_pos.x, slingshot_pos.y + 20.0, 2.0),
        RigidBody::Kinematic,   // Kinematic while waiting
        Collider::circle(29.0), // ~59px width / 2
        Bird,
    ));

    // Complex Tower
    spawn_complex_tower(&mut commands, &asset_server, Vec2::new(200.0, -250.0));
}

fn spawn_complex_tower(commands: &mut Commands, asset_server: &Res<AssetServer>, pos: Vec2) {
    let stone_sq_size = 80.0;
    let wood_rect_w = 72.0;
    let wood_rect_h = 22.0;
    let glass_w = 52.0;
    let glass_h = 72.0;

    // --- Base Layer (Stone) ---
    // Left Pillar
    commands.spawn((
        Sprite::from_image(asset_server.load("block_stone_square.png")),
        Transform::from_xyz(pos.x, pos.y, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(stone_sq_size, stone_sq_size),
        Block,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("block_stone_square.png")),
        Transform::from_xyz(pos.x, pos.y + stone_sq_size, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(stone_sq_size, stone_sq_size),
        Block,
    ));

    // Right Pillar
    commands.spawn((
        Sprite::from_image(asset_server.load("block_stone_square.png")),
        Transform::from_xyz(pos.x + 200.0, pos.y, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(stone_sq_size, stone_sq_size),
        Block,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("block_stone_square.png")),
        Transform::from_xyz(pos.x + 200.0, pos.y + stone_sq_size, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(stone_sq_size, stone_sq_size),
        Block,
    ));

    // Pig in bottom middle
    commands.spawn((
        Sprite::from_image(asset_server.load("pig_green.png")),
        Transform::from_xyz(pos.x + 100.0, pos.y, 0.0),
        RigidBody::Dynamic,
        Collider::circle(26.0),
        Pig,
    ));

    // --- First Floor (Wood Planks) ---
    let floor1_y = pos.y + stone_sq_size * 1.5 + 10.0;
    // Long plank across (composed of multiple small ones since we only have small rects)
    for i in 0..4 {
        commands.spawn((
            Sprite::from_image(asset_server.load("block_wood_rect.png")),
            Transform::from_xyz(pos.x + (i as f32 * wood_rect_w), floor1_y, 0.0),
            RigidBody::Dynamic,
            Collider::rectangle(wood_rect_w, wood_rect_h),
            Block,
        ));
    }

    // --- Second Floor (Glass & Wood) ---
    let floor2_y = floor1_y + glass_h / 2.0 + 10.0;

    // Glass blocks
    commands.spawn((
        Sprite::from_image(asset_server.load("block_glass_square.png")),
        Transform::from_xyz(pos.x + 50.0, floor2_y, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(glass_w, glass_h),
        Block,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("block_glass_square.png")),
        Transform::from_xyz(pos.x + 150.0, floor2_y, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(glass_w, glass_h),
        Block,
    ));

    // Pig in middle
    commands.spawn((
        Sprite::from_image(asset_server.load("pig_green.png")),
        Transform::from_xyz(pos.x + 100.0, floor2_y, 0.0),
        RigidBody::Dynamic,
        Collider::circle(26.0),
        Pig,
    ));

    // --- Roof (Wood) ---
    let roof_y = floor2_y + glass_h / 2.0 + 10.0;
    for i in 0..3 {
        commands.spawn((
            Sprite::from_image(asset_server.load("block_wood_rect.png")),
            Transform::from_xyz(pos.x + 40.0 + (i as f32 * wood_rect_w), roof_y, 0.0),
            RigidBody::Dynamic,
            Collider::rectangle(wood_rect_w, wood_rect_h),
            Block,
        ));
    }

    // Top Blocks
    commands.spawn((
        Sprite::from_image(asset_server.load("block_wood_square.png")),
        Transform::from_xyz(pos.x + 100.0, roof_y + 50.0, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(88.0, 89.0),
        Block,
    ));

    // Top Pig
    commands.spawn((
        Sprite::from_image(asset_server.load("pig_green.png")),
        Transform::from_xyz(pos.x + 100.0, roof_y + 120.0, 0.0),
        RigidBody::Dynamic,
        Collider::circle(26.0),
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
