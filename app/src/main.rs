use crate::camera::Camera;
use eframe::{egui, wgpu};
use std::time::Instant;

pub mod camera;

struct App {
    last_time: Option<Instant>,

    info_window_open: bool,

    camera_window_open: bool,
    camera: Camera,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            last_time: None,

            info_window_open: true,

            camera_window_open: true,
            camera: Camera::new(cgmath::Vector4 {
                x: -3.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let time = Instant::now();
        let dt = (time - self.last_time.unwrap_or(time)).as_secs_f32();
        self.last_time = Some(time);

        egui::TopBottomPanel::top("Windows").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.info_window_open |= ui.button("Info").clicked();
                self.camera_window_open |= ui.button("Camera").clicked();
            });
        });

        egui::Window::new("Info")
            .open(&mut self.info_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt));
                ui.label(format!("Frame Time: {:.3}ms", 1000.0 * dt));
                ui.allocate_space(ui.available_size());
            });

        egui::Window::new("Camera")
            .open(&mut self.camera_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui_vector4(ui, &mut self.camera.position);
                });
                ui.add_enabled_ui(false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Forward:");
                        ui_vector4(ui, &mut self.camera.rotation().forward());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Up:");
                        ui_vector4(ui, &mut self.camera.rotation().up());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Right:");
                        ui_vector4(ui, &mut self.camera.rotation().right());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Ana:");
                        ui_vector4(ui, &mut self.camera.rotation().ana());
                    });
                });
                ui.allocate_space(ui.available_size());
            });

        if !ctx.wants_keyboard_input() && !ctx.wants_pointer_input() {
            ctx.input(|i| self.camera.update(dt, i));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                _ = ui;
            });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "4d Rendering",
        eframe::NativeOptions {
            vsync: false,
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                present_mode: wgpu::PresentMode::AutoNoVsync,
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

fn ui_vector4(
    ui: &mut egui::Ui,
    cgmath::Vector4 { x, y, z, w }: &mut cgmath::Vector4<f32>,
) -> egui::Response {
    ui.add(egui::DragValue::new(x).speed(0.1).prefix("x:"))
        | ui.add(egui::DragValue::new(y).speed(0.1).prefix("y:"))
        | ui.add(egui::DragValue::new(z).speed(0.1).prefix("z:"))
        | ui.add(egui::DragValue::new(w).speed(0.1).prefix("w:"))
}
