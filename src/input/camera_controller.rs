use crate::assets::camera::Camera;
use crate::core::math::Vec3;
use std::f32::consts::FRAC_PI_2;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy)]
pub struct FreeCameraControllerDescriptor {
    pub speed: f32,
    pub sensitivity: f32,
    pub scroll_line_speed: f32,
    pub scroll_pixel_speed: f32,
    pub fov_zoom_speed: f32,
    pub min_fov: f32,
    pub max_fov: f32,
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
        }
    }
}

pub struct FreeCameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub scroll_line_speed: f32,
    pub scroll_pixel_speed: f32,
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

impl FreeCameraController {
    pub fn new(desc: FreeCameraControllerDescriptor) -> Self {
        Self {
            speed: desc.speed,
            sensitivity: desc.sensitivity,
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

    pub fn process_keyboard(&mut self, event: &KeyEvent) -> bool {
        let amount = if event.state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        if let PhysicalKey::Code(key_code) = event.physical_key {
            match key_code {
                KeyCode::KeyW | KeyCode::ArrowUp => {
                    self.amount_forward = amount;
                    true
                }
                KeyCode::KeyS | KeyCode::ArrowDown => {
                    self.amount_backward = amount;
                    true
                }
                KeyCode::KeyA | KeyCode::ArrowLeft => {
                    self.amount_left = amount;
                    true
                }
                KeyCode::KeyD | KeyCode::ArrowRight => {
                    self.amount_right = amount;
                    true
                }
                KeyCode::Space => {
                    self.amount_up = amount;
                    true
                }
                KeyCode::ShiftLeft => {
                    self.amount_down = amount;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn process_mouse_button(&mut self, button: winit::event::MouseButton, state: ElementState) {
        if button == winit::event::MouseButton::Left {
            self.is_mouse_pressed = state == ElementState::Pressed;
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        if self.is_mouse_pressed {
            self.rotate_horizontal = mouse_dx as f32;
            self.rotate_vertical = mouse_dy as f32;
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * self.scroll_line_speed,
            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32 * self.scroll_pixel_speed,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: std::time::Duration) {
        let dt = dt.as_secs_f32();

        self.yaw += self.rotate_horizontal * self.sensitivity * dt;
        self.pitch += -self.rotate_vertical * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        if self.pitch < -FRAC_PI_2 + 0.001 {
            self.pitch = -FRAC_PI_2 + 0.001;
        }
        if self.pitch > FRAC_PI_2 - 0.001 {
            self.pitch = FRAC_PI_2 - 0.001;
        }

        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let forward = Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = Vec3::Y;

        let move_dir = forward * (self.amount_forward - self.amount_backward)
            + right * (self.amount_right - self.amount_left)
            + up * (self.amount_up - self.amount_down);

        if move_dir.length_squared() > 0.0 {
            camera.position += move_dir.normalize() * self.speed * dt;
        }

        camera.target = camera.position + forward;

        camera.fov += self.scroll * self.speed * dt * self.fov_zoom_speed;
        camera.fov = camera.fov.clamp(self.min_fov, self.max_fov);
        self.scroll = 0.0;
    }
}
