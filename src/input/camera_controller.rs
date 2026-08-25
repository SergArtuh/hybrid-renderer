use crate::assets::camera::Camera;
use crate::core::math::Vec3;
use std::f32::consts::FRAC_PI_2;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy)]
pub struct OrbitCameraControllerDescriptor {
    pub target: Vec3,
    pub distance: f32,
    pub sensitivity: f32,
    pub scroll_speed: f32,
    pub pinch_zoom_speed: f32,
    pub move_speed: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub start_yaw: f32,
    pub start_pitch: f32,
}

impl Default for OrbitCameraControllerDescriptor {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 10.0,
            sensitivity: 1.0,
            scroll_speed: 5.0,
            pinch_zoom_speed: 1.0,
            move_speed: 5.0,
            min_distance: 0.1,
            max_distance: 100.0,
            min_pitch: -FRAC_PI_2 + 0.1,
            max_pitch: FRAC_PI_2 - 0.1,
            start_yaw: -FRAC_PI_2,
            start_pitch: 0.4,
        }
    }
}

pub struct OrbitCameraController {
    pub sensitivity: f32,
    pub scroll_speed: f32,
    pub pinch_zoom_speed: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,

    target: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,

    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    is_mouse_pressed: bool,
}

impl OrbitCameraController {
    pub fn new(desc: OrbitCameraControllerDescriptor) -> Self {
        Self {
            sensitivity: desc.sensitivity,
            scroll_speed: desc.scroll_speed,
            pinch_zoom_speed: desc.pinch_zoom_speed,
            min_distance: desc.min_distance,
            max_distance: desc.max_distance,
            min_pitch: desc.min_pitch,
            max_pitch: desc.max_pitch,
            target: desc.target,
            distance: desc.distance,
            yaw: desc.start_yaw,
            pitch: desc.start_pitch,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            is_mouse_pressed: false,
        }
    }
}

pub struct FreeCameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub scroll_line_speed: f32,
    pub scroll_pixel_speed: f32,
    pub pinch_zoom_speed: f32,
    pub fov_zoom_speed: f32,
    pub min_fov: f32,
    pub max_fov: f32,

    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,

    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,

    yaw: f32,
    pitch: f32,
    is_mouse_pressed: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct FreeCameraControllerDescriptor {
    pub speed: f32,
    pub sensitivity: f32,
    pub scroll_line_speed: f32,
    pub scroll_pixel_speed: f32,
    pub pinch_zoom_speed: f32,
    pub fov_zoom_speed: f32,
    pub min_fov: f32,
    pub max_fov: f32,
}

impl FreeCameraController {
    pub fn new(desc: FreeCameraControllerDescriptor) -> Self {
        Self {
            speed: desc.speed,
            sensitivity: desc.sensitivity,
            pinch_zoom_speed: desc.pinch_zoom_speed,
            scroll_line_speed: desc.scroll_line_speed,
            scroll_pixel_speed: desc.scroll_pixel_speed,
            fov_zoom_speed: desc.fov_zoom_speed,
            min_fov: desc.min_fov,
            max_fov: desc.max_fov,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            yaw: -FRAC_PI_2,
            pitch: 0.0,
            is_mouse_pressed: false,
        }
    }
}

pub enum CameraController {
    Orbit(OrbitCameraController),
    Free(FreeCameraController),
}

pub struct OrbitCameraControllerSystem;

impl OrbitCameraControllerSystem {
    pub fn process_keyboard(_controller: &mut OrbitCameraController, _event: &KeyEvent) -> bool {
        true // no- op
    }

    pub fn process_mouse_button(
        controller: &mut OrbitCameraController,
        button: winit::event::MouseButton,
        state: ElementState,
    ) {
        if button == winit::event::MouseButton::Left {
            controller.is_mouse_pressed = state == ElementState::Pressed;
        }
    }

    pub fn process_mouse(controller: &mut OrbitCameraController, mouse_dx: f64, mouse_dy: f64) {
        if controller.is_mouse_pressed {
            controller.rotate_horizontal += mouse_dx as f32;
            controller.rotate_vertical += mouse_dy as f32;
        }
    }

    pub fn process_scroll(controller: &mut OrbitCameraController, delta: &MouseScrollDelta) {
        controller.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll,
            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32 * 0.01,
        };
    }

    pub fn process_pinch(controller: &mut OrbitCameraController, delta: &MouseScrollDelta) {
        controller.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * controller.pinch_zoom_speed,
            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32 * controller.pinch_zoom_speed,
        };
    }

    // pub fn set_target(controller: &mut OrbitCameraController, target: Vec3) {
    //     controller.target = target;
    // }

    fn update_camera(
        controller: &mut OrbitCameraController,
        camera: &mut Camera,
        dt: std::time::Duration,
    ) {
        let dt = dt.as_secs_f32();

        controller.yaw += controller.rotate_horizontal * controller.sensitivity * dt;
        controller.pitch += controller.rotate_vertical * controller.sensitivity * dt;

        controller.rotate_horizontal = 0.0;
        controller.rotate_vertical = 0.0;

        controller.pitch = controller
            .pitch
            .clamp(controller.min_pitch, controller.max_pitch);

        let zoom_delta =
            controller.scroll * controller.scroll_speed * dt * controller.distance.max(1.0);
        controller.distance = (controller.distance + zoom_delta)
            .clamp(controller.min_distance, controller.max_distance);
        controller.scroll = 0.0;

        let (sin_pitch, cos_pitch) = controller.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = controller.yaw.sin_cos();

        let offset = Vec3::new(
            controller.distance * cos_pitch * cos_yaw,
            controller.distance * sin_pitch,
            controller.distance * cos_pitch * sin_yaw,
        );

        camera.position = controller.target + offset;
        camera.target = controller.target;
    }
}

impl Default for FreeCameraControllerDescriptor {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sensitivity: 2.0,
            scroll_line_speed: 5.0,
            scroll_pixel_speed: 0.05,
            fov_zoom_speed: 10.0,
            min_fov: 10.0,
            max_fov: 120.0,
            pinch_zoom_speed: 1.0,
        }
    }
}

pub struct FreeCameraControllerSystem;

impl FreeCameraControllerSystem {
    pub fn process_keyboard(controller: &mut FreeCameraController, event: &KeyEvent) -> bool {
        let amount = if event.state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        if let PhysicalKey::Code(key_code) = event.physical_key {
            match key_code {
                KeyCode::KeyW | KeyCode::ArrowUp => {
                    controller.amount_forward = amount;
                    true
                }
                KeyCode::KeyS | KeyCode::ArrowDown => {
                    controller.amount_backward = amount;
                    true
                }
                KeyCode::KeyA | KeyCode::ArrowLeft => {
                    controller.amount_left = amount;
                    true
                }
                KeyCode::KeyD | KeyCode::ArrowRight => {
                    controller.amount_right = amount;
                    true
                }
                KeyCode::Space => {
                    controller.amount_up = amount;
                    true
                }
                KeyCode::ShiftLeft => {
                    controller.amount_down = amount;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn process_mouse_button(
        controller: &mut FreeCameraController,
        button: winit::event::MouseButton,
        state: ElementState,
    ) {
        if button == winit::event::MouseButton::Left {
            controller.is_mouse_pressed = state == ElementState::Pressed;
        }
    }

    pub fn process_mouse(controller: &mut FreeCameraController, mouse_dx: f64, mouse_dy: f64) {
        if controller.is_mouse_pressed {
            controller.rotate_horizontal += mouse_dx as f32;
            controller.rotate_vertical += mouse_dy as f32;
        }
    }

    pub fn process_scroll(controller: &mut FreeCameraController, delta: &MouseScrollDelta) {
        controller.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * controller.scroll_line_speed,
            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32 * controller.scroll_pixel_speed,
        };
    }

    pub fn process_pinch(controller: &mut FreeCameraController, delta: &MouseScrollDelta) {
        controller.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * controller.pinch_zoom_speed,
            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32 * controller.pinch_zoom_speed,
        };
    }

    pub fn update_camera(
        controller: &mut FreeCameraController,
        camera: &mut Camera,
        dt: std::time::Duration,
    ) {
        let dt = dt.as_secs_f32();

        controller.yaw += controller.rotate_horizontal * controller.sensitivity * dt;
        controller.pitch += -controller.rotate_vertical * controller.sensitivity * dt;

        controller.rotate_horizontal = 0.0;
        controller.rotate_vertical = 0.0;

        if controller.pitch < -FRAC_PI_2 + 0.001 {
            controller.pitch = -FRAC_PI_2 + 0.001;
        }
        if controller.pitch > FRAC_PI_2 - 0.001 {
            controller.pitch = FRAC_PI_2 - 0.001;
        }

        let (sin_pitch, cos_pitch) = controller.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = controller.yaw.sin_cos();
        let forward = Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = Vec3::Y;

        let move_dir = forward * (controller.amount_forward - controller.amount_backward)
            + right * (controller.amount_right - controller.amount_left)
            + up * (controller.amount_up - controller.amount_down);

        if move_dir.length_squared() > 0.0 {
            camera.position += move_dir.normalize() * controller.speed * dt;
        }

        camera.target = camera.position + forward;

        camera.fov += controller.scroll * controller.speed * dt * controller.fov_zoom_speed;
        camera.fov = camera.fov.clamp(controller.min_fov, controller.max_fov);
        controller.scroll = 0.0;
    }
}

pub enum CameraControllerSystem {
    Orbit(OrbitCameraControllerSystem),
    Free(FreeCameraControllerSystem),
}

impl CameraControllerSystem {
    pub fn process_keyboard(controller: &mut CameraController, event: &KeyEvent) -> bool {
        match controller {
            CameraController::Orbit(controller) => {
                OrbitCameraControllerSystem::process_keyboard(controller, event)
            }
            CameraController::Free(controller) => {
                FreeCameraControllerSystem::process_keyboard(controller, event)
            }
        }
    }

    pub fn process_mouse_button(
        controller: &mut CameraController,
        button: winit::event::MouseButton,
        state: ElementState,
    ) {
        match controller {
            CameraController::Orbit(controller) => {
                OrbitCameraControllerSystem::process_mouse_button(controller, button, state)
            }
            CameraController::Free(controller) => {
                FreeCameraControllerSystem::process_mouse_button(controller, button, state)
            }
        }
    }

    pub fn process_mouse(controller: &mut CameraController, mouse_dx: f64, mouse_dy: f64) {
        match controller {
            CameraController::Orbit(controller) => {
                OrbitCameraControllerSystem::process_mouse(controller, mouse_dx, mouse_dy)
            }
            CameraController::Free(controller) => {
                FreeCameraControllerSystem::process_mouse(controller, mouse_dx, mouse_dy)
            }
        }
    }

    pub fn process_scroll(controller: &mut CameraController, delta: &MouseScrollDelta) {
        match controller {
            CameraController::Orbit(controller) => {
                OrbitCameraControllerSystem::process_scroll(controller, delta)
            }
            CameraController::Free(controller) => {
                FreeCameraControllerSystem::process_scroll(controller, delta)
            }
        }
    }

    pub fn process_pinch(controller: &mut CameraController, delta: &MouseScrollDelta) {
        match controller {
            CameraController::Orbit(controller) => {
                OrbitCameraControllerSystem::process_pinch(controller, delta)
            }
            CameraController::Free(controller) => {
                FreeCameraControllerSystem::process_pinch(controller, delta)
            }
        }
    }

    pub fn update_camera(
        controller: &mut CameraController,
        camera: &mut Camera,
        dt: std::time::Duration,
    ) {
        match controller {
            CameraController::Orbit(controller) => {
                OrbitCameraControllerSystem::update_camera(controller, camera, dt)
            }
            CameraController::Free(controller) => {
                FreeCameraControllerSystem::update_camera(controller, camera, dt)
            }
        }
    }
}
