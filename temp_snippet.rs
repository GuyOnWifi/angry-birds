fn configure_view_system(mut q: Query<&mut OrthographicProjection, With<Camera2d>>) {
    // Zoom out so everything fits
    for mut proj in &mut q {
        proj.scaling_mode = bevy::render::camera::ScalingMode::AutoMin {
            min_width: 1200.0,
            min_height: 800.0,
        };
    }
}
