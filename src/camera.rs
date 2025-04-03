use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::EguiContext;

#[derive(Component)]
pub struct CameraController {
    pub orbit_speed: f32,
    pub pan_speed: f32,
    pub zoom_speed: f32,
    pub key_move_speed: f32,
    pub orbit_button: MouseButton,
    pub pan_button: MouseButton,
    pub distance: f32,
    pub target: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            orbit_speed: 1.0,
            pan_speed: 1.0,
            zoom_speed: 1.0,
            key_move_speed: 2.0,
            orbit_button: MouseButton::Left,
            pan_button: MouseButton::Right,
            distance: 5.0,
            target: Vec3::new(0.0, 0.5, 0.0),
        }
    }
}

pub fn setup_camera(mut commands: Commands) {
    // Set up camera with top-down view
    let target = Vec3::ZERO;
    let distance = 3.0; // Suitable for -1~1 coordinate range

    let controller = CameraController {
        distance,
        target,
        key_move_speed: 0.5, // Lower speed for small coordinate range
        ..default()
    };

    // Position camera above and slightly to the side
    let position = Vec3::new(distance * 0.5, distance * 0.8, distance * 0.6);

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(position).looking_at(target, Vec3::Y),
        controller,
    ));

    // Add directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 2.0, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn camera_controller_system(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
    mut egui_context: Query<&mut EguiContext>,
) {
    // https://docs.rs/bevy_egui/latest/bevy_egui/struct.EguiContext.html#method.get
    // Check if UI is handling input
    if let Ok(mut context) = egui_context.get_single_mut() {
        // Get mutable reference
        if context.get_mut().is_using_pointer() {
            // Clear mouse events when UI is being used
            mouse_motion.clear();
            mouse_wheel.clear();
            return;
        }
    }

    let _window = window_q.single();

    for (mut transform, mut controller) in query.iter_mut() {
        // Orbit rotation (left mouse button)
        if mouse_buttons.pressed(controller.orbit_button) {
            for ev in mouse_motion.read() {
                let delta_x = ev.delta.x * controller.orbit_speed * 0.01;
                let delta_y = ev.delta.y * controller.orbit_speed * 0.01;

                // Get direction vector from camera to target
                let look_dir = (controller.target - transform.translation).normalize();

                // Calculate right and up vectors
                let right = look_dir.cross(Vec3::Y).normalize();

                // Rotate around target point
                let rotation_y = Quat::from_axis_angle(Vec3::Y, -delta_x);

                // Calculate current angle with vertical axis
                let camera_to_target = transform.translation - controller.target;
                let current_angle = Vec3::Y.angle_between(camera_to_target);

                // Limit vertical rotation to prevent extreme angles
                // Allow rotation within 1° to 179° (almost full range but avoiding extremes)
                let min_angle = 1.0f32.to_radians();
                let max_angle = 179.0f32.to_radians();

                // Calculate new angle after rotation
                let new_angle = current_angle - delta_y;

                // Apply vertical rotation only if it stays within limits
                let rotation_x = if new_angle >= min_angle && new_angle <= max_angle {
                    Quat::from_axis_angle(right, -delta_y)
                } else {
                    // Skip vertical rotation if it would exceed limits
                    Quat::IDENTITY
                };

                let mut camera_to_target = transform.translation - controller.target;

                // Apply rotation
                camera_to_target = rotation_y * rotation_x * camera_to_target;

                // Update camera position
                transform.translation = controller.target + camera_to_target;

                // Keep camera looking at target
                transform.look_at(controller.target, Vec3::Y);
            }
        }

        // Panning (right mouse button)
        if mouse_buttons.pressed(controller.pan_button) {
            for ev in mouse_motion.read() {
                let delta_x = ev.delta.x * controller.pan_speed * 0.005;
                let delta_y = ev.delta.y * controller.pan_speed * 0.005;

                let right = transform.right();
                let up = transform.up();

                let translation = right * -delta_x + up * delta_y;

                transform.translation += translation;
                controller.target += translation;
            }
        }

        // Zoom (mouse wheel)
        for ev in mouse_wheel.read() {
            let zoom_amount = -ev.y * controller.zoom_speed * controller.distance * 0.1;

            let direction = (transform.translation - controller.target).normalize();

            controller.distance += zoom_amount;
            controller.distance = controller.distance.max(0.5);

            transform.translation = controller.target + direction * controller.distance;
        }

        // Keyboard movement
        let mut keyboard_translation = Vec3::ZERO;

        // Forward/Backward (W/S)
        if keys.pressed(KeyCode::KeyW) {
            let forward = (controller.target - transform.translation).normalize();
            let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
            keyboard_translation += horizontal_forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            let forward = (controller.target - transform.translation).normalize();
            let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
            keyboard_translation -= horizontal_forward;
        }

        // Up/Down (Q/E)
        if keys.pressed(KeyCode::KeyQ) {
            keyboard_translation.y += 1.0; // Move up
        }
        if keys.pressed(KeyCode::KeyE) {
            keyboard_translation.y -= 1.0; // Move down
        }

        // Left/Right (A/D)
        if keys.pressed(KeyCode::KeyA) {
            let right = transform.right();
            let horizontal_right = Vec3::new(right.x, 0.0, right.z).normalize();
            keyboard_translation -= horizontal_right;
        }
        if keys.pressed(KeyCode::KeyD) {
            let right = transform.right();
            let horizontal_right = Vec3::new(right.x, 0.0, right.z).normalize();
            keyboard_translation += horizontal_right;
        }

        // Apply keyboard translation
        if keyboard_translation != Vec3::ZERO {
            if keyboard_translation.length_squared() > 0.0 {
                keyboard_translation = keyboard_translation.normalize();
            }
            let translation = keyboard_translation * controller.key_move_speed * time.delta_secs();

            transform.translation += translation;
            controller.target += translation;
        }
    }
}
