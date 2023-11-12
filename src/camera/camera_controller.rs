use std::time::Instant;

use crate::{input_event::{input_service::{InputService, Key}, KeypressState}, my_time::Time};
use nalgebra_glm as glm;

use super::camera::Camera;

pub struct CameraController {
    yaw: f32,
    pitch: f32,
    camera: Camera
}


impl CameraController {
    const SPEED: f32 = 14.0;
    const SENSETIV: f32 = 0.3;
    pub fn new(position: glm::Vec3, fov: f32) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            camera: Camera::new(position, fov)
        }
    }

    pub fn update(&mut self, input: &InputService, delta: f32, is_cursor: bool) {
        if !is_cursor {
            self.camera.rotation = glm::Mat4::identity();
            self.yaw -= input.delta().0*Self::SENSETIV*delta;
            self.pitch -= input.delta().1*Self::SENSETIV*delta;
            if self.pitch > 1.569_051 {self.pitch = 1.569_051}
            if self.pitch < -1.569_051 {self.pitch = -1.569_051}
            self.camera.rotate(self.pitch, self.yaw, 0.0); 
        }

        if input.is_key(&Key::W, KeypressState::AnyStayPress) {
            self.camera.position +=  self.camera.front * Self::SPEED * delta;
        }
        if input.is_key(&Key::S, KeypressState::AnyStayPress) {
            self.camera.position -=  self.camera.front * Self::SPEED * delta;
        }
        if input.is_key(&Key::A, KeypressState::AnyStayPress) {
            self.camera.position -=  self.camera.right * Self::SPEED * delta;
        }
        if input.is_key(&Key::D, KeypressState::AnyStayPress) {
            self.camera.position +=  self.camera.right * Self::SPEED * delta;
        }
    }

    pub fn projection(&self, width: f32, height: f32) -> glm::Mat4 {
        self.camera.projection(width, height)
    }
    pub fn view(&self) -> glm::Mat4 {
        self.camera.view()
    }
    pub fn proj_view(&self, width: f32, height: f32) -> glm::Mat4 {
        self.camera.proj_view(width, height)
    }
    pub fn position(&self) -> &glm::Vec3 {&self.camera.position()}
    pub fn front(&self) -> &glm::Vec3 {&self.camera.front()}
    pub fn position_array(&self) -> [f32; 3] {self.camera.position_array()}
    pub fn front_array(&self) -> [f32; 3] {self.camera.front_array()}
}