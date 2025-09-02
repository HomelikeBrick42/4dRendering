use eframe::egui;
use math::{Rotor, Transform};
use std::f32::consts::TAU;

pub struct Camera {
    pub position: cgmath::Vector4<f32>,
    pub main_rotation: Rotor,
    pub xy_rotation: f32,

    pub move_speed: f32,
    pub rotation_speed: f32,
}

impl Camera {
    pub fn new(position: cgmath::Vector4<f32>) -> Self {
        Self {
            position,
            main_rotation: Rotor::identity(),
            xy_rotation: 0.0,

            move_speed: 2.0,
            rotation_speed: 0.5,
        }
    }

    pub fn rotation(&self) -> Rotor {
        self.main_rotation.then(Rotor::rotate_xy(self.xy_rotation))
    }

    pub fn transform(&self) -> Transform {
        Transform::translation(self.position).then(Transform::from_rotor(self.rotation()))
    }

    pub fn update(&mut self, ts: f32, i: &egui::InputState) {
        let mut move_speed = self.move_speed;
        let rotation_speed = self.rotation_speed * TAU;

        if i.modifiers.shift {
            move_speed *= 2.0;
        }

        {
            let rotation = self.rotation();
            let forward = rotation.x();
            let up = rotation.y();
            let right = rotation.z();
            let ana = rotation.w();

            if i.key_down(egui::Key::W) {
                self.position += forward * move_speed * ts;
            }
            if i.key_down(egui::Key::S) {
                self.position -= forward * move_speed * ts;
            }
            if i.key_down(egui::Key::E) {
                self.position += up * move_speed * ts;
            }
            if i.key_down(egui::Key::Q) {
                self.position -= up * move_speed * ts;
            }
            if i.key_down(egui::Key::D) {
                self.position += right * move_speed * ts;
            }
            if i.key_down(egui::Key::A) {
                self.position -= right * move_speed * ts;
            }
            if i.key_down(egui::Key::R) {
                self.position += ana * move_speed * ts;
            }
            if i.key_down(egui::Key::F) {
                self.position -= ana * move_speed * ts;
            }
        }

        if i.modifiers.ctrl {
            if i.key_down(egui::Key::ArrowRight) {
                self.main_rotation = self
                    .main_rotation
                    .then(Rotor::rotate_xw(rotation_speed * ts));
            }
            if i.key_down(egui::Key::ArrowLeft) {
                self.main_rotation = self
                    .main_rotation
                    .then(Rotor::rotate_xw(-rotation_speed * ts));
            }

            if i.key_down(egui::Key::ArrowUp) {
                self.main_rotation = self
                    .main_rotation
                    .then(Rotor::rotate_zw(rotation_speed * ts));
            }
            if i.key_down(egui::Key::ArrowDown) {
                self.main_rotation = self
                    .main_rotation
                    .then(Rotor::rotate_zw(-rotation_speed * ts));
            }
        } else {
            if i.key_down(egui::Key::ArrowRight) {
                self.main_rotation = self
                    .main_rotation
                    .then(Rotor::rotate_xz(rotation_speed * ts));
            }
            if i.key_down(egui::Key::ArrowLeft) {
                self.main_rotation = self
                    .main_rotation
                    .then(Rotor::rotate_xz(-rotation_speed * ts));
            }

            if i.key_down(egui::Key::ArrowUp) {
                self.xy_rotation += rotation_speed * ts;
            }
            if i.key_down(egui::Key::ArrowDown) {
                self.xy_rotation -= rotation_speed * ts;
            }
        }

        self.xy_rotation = self.xy_rotation.clamp(-TAU * 0.25, TAU * 0.25);
    }
}
