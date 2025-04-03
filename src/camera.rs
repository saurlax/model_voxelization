use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

// 相机控制组件
#[derive(Component)]
pub struct CameraController {
    pub orbit_speed: f32,
    pub pan_speed: f32,
    pub zoom_speed: f32,
    pub key_move_speed: f32, // 添加键盘移动速度控制
    pub orbit_button: MouseButton,
    pub pan_button: MouseButton,
    pub distance: f32,
    // 添加目标点来跟踪相机焦点
    pub target: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            orbit_speed: 1.0,
            pan_speed: 1.0,
            zoom_speed: 1.0,
            key_move_speed: 2.0, // 键盘移动速度
            orbit_button: MouseButton::Left,
            pan_button: MouseButton::Right,
            distance: 5.0,                    // 初始相机距离
            target: Vec3::new(0.0, 0.5, 0.0), // 初始目标点
        }
    }
}

pub fn setup_camera(mut commands: Commands) {
    // 将相机设置为从上方俯视模型的位置
    let target = Vec3::ZERO; // 目标点在原点
    let distance = 48.0; // 足够的距离以查看整个体素空间(-32~32)

    let controller = CameraController {
        distance,
        target,
        // 增加键盘移动速度，方便快速浏览
        key_move_speed: 5.0,
        ..default()
    };

    // 从上方向下看的相机位置
    let position = Vec3::new(distance * 0.5, distance * 0.8, distance * 0.6);

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(position).looking_at(target, Vec3::Y),
        controller,
    ));

    // 添加环境光，调整位置以更好地照亮模型
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 40.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn camera_controller_system(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>, // 添加键盘输入
    time: Res<Time>,                 // 添加时间资源以实现平滑移动
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
) {
    let _window = window_q.single();

    for (mut transform, mut controller) in query.iter_mut() {
        // 轨道旋转 (左键)
        if mouse_buttons.pressed(controller.orbit_button) {
            for ev in mouse_motion.read() {
                let delta_x = ev.delta.x * controller.orbit_speed * 0.01;
                let delta_y = ev.delta.y * controller.orbit_speed * 0.01;

                // 获取从相机到目标的方向向量
                let look_dir = (controller.target - transform.translation).normalize();

                // 计算相机的右向量和上向量
                let right = look_dir.cross(Vec3::Y).normalize();
                let up = right.cross(look_dir).normalize();

                // 围绕目标点旋转
                // 围绕Y轴旋转
                let rotation_y = Quat::from_axis_angle(Vec3::Y, -delta_x);

                // 围绕右轴旋转
                let rotation_x = Quat::from_axis_angle(right, -delta_y);

                // 计算相机到目标的向量
                let mut camera_to_target = transform.translation - controller.target;

                // 应用旋转
                camera_to_target = rotation_y * rotation_x * camera_to_target;

                // 更新相机位置
                transform.translation = controller.target + camera_to_target;

                // 保持相机朝向目标点
                transform.look_at(controller.target, Vec3::Y);
            }
        }

        // 平移 (右键)
        if mouse_buttons.pressed(controller.pan_button) {
            for ev in mouse_motion.read() {
                let delta_x = ev.delta.x * controller.pan_speed * 0.005;
                let delta_y = ev.delta.y * controller.pan_speed * 0.005;

                // 获取相机的右方向和上方向
                let right = transform.right();
                let up = transform.up();

                // 计算平移向量
                let translation = right * -delta_x + up * delta_y;

                // 同时移动相机和目标点，保持相对关系
                transform.translation += translation;
                controller.target += translation;
            }
        }

        // 缩放 (滚轮) - 修复方向问题
        for ev in mouse_wheel.read() {
            // 改进缩放逻辑: 向上滚动(正值)应该缩小距离，向下滚动(负值)应该增加距离
            let zoom_amount = -ev.y * controller.zoom_speed * controller.distance * 0.1;

            // 计算相机到目标的方向
            let direction = (transform.translation - controller.target).normalize();

            // 更新距离，并设置最小值避免过度缩放
            controller.distance += zoom_amount;
            controller.distance = controller.distance.max(0.5);

            // 根据新的距离和方向更新相机位置
            transform.translation = controller.target + direction * controller.distance;
        }

        // 修改键盘控制
        let mut keyboard_translation = Vec3::ZERO;

        // 前后移动 (W/S)
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

        // 上下移动 (Q/E)
        if keys.pressed(KeyCode::KeyQ) {
            // 向上移动 (Y轴正方向)
            keyboard_translation.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyE) {
            // 向下移动 (Y轴负方向)
            keyboard_translation.y -= 1.0;
        }

        // 左右移动 (A/D)
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

        // 应用键盘平移
        if keyboard_translation != Vec3::ZERO {
            // 归一化向量并应用速度和帧时间
            if keyboard_translation.length_squared() > 0.0 {
                keyboard_translation = keyboard_translation.normalize();
            }
            let translation = keyboard_translation * controller.key_move_speed * time.delta_secs();

            // 同时移动相机和目标点，保持相对关系
            transform.translation += translation;
            controller.target += translation;
        }
    }
}
