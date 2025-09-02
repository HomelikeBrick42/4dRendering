use crate::camera::Camera;
use eframe::{egui, wgpu};
use rendering::{HyperSphere, RenderData, RenderState, ViewAxes, register_rendering_state};
use std::{sync::Arc, time::Instant};

pub mod camera;

struct App {
    last_time: Option<Instant>,

    info_window_open: bool,

    camera_window_open: bool,
    camera: Camera,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        register_rendering_state(cc);

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
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let eframe::egui_wgpu::RenderState {
            device,
            queue,
            renderer,
            ..
        } = frame.wgpu_render_state().unwrap();

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
                ui.horizontal(|ui| {
                    ui.label("Move Speed:");
                    ui.add(egui::DragValue::new(&mut self.camera.move_speed).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Rotation Speed:");
                    ui.add(egui::DragValue::new(&mut self.camera.rotation_speed).speed(0.1));
                    self.camera.rotation_speed = self.camera.rotation_speed.max(0.0);
                });
                ui.add_enabled_ui(false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Forward:");
                        ui_vector4(ui, &mut self.camera.rotation().x());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Up:");
                        ui_vector4(ui, &mut self.camera.rotation().y());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Right:");
                        ui_vector4(ui, &mut self.camera.rotation().z());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Ana:");
                        ui_vector4(ui, &mut self.camera.rotation().w());
                    });
                });
                ui.allocate_space(ui.available_size());
            });

        if !ctx.wants_keyboard_input() && !ctx.wants_pointer_input() {
            ctx.input(|i| self.camera.update(dt, i));
        }

        {
            let callback_resources = &mut renderer.write().callback_resources;
            let render_state: &mut RenderState = callback_resources.get_mut().unwrap();

            render_state.update_hyper_spheres(
                device,
                queue,
                &[HyperSphere {
                    position: cgmath::Vector4 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    color: cgmath::Vector3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    radius: 1.0,
                }],
            );
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let (rect, _response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::all());

                ui.painter()
                    .add(eframe::egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RenderData {
                            camera_transform: self.camera.transform(),
                            view_axes: ViewAxes::XYZ,
                            aspect: rect.width() / rect.height(),
                        },
                    ));
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
                wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
                    eframe::egui_wgpu::WgpuSetupCreateNew {
                        device_descriptor: Arc::new(|adapter| wgpu::DeviceDescriptor {
                            label: Some("Device"),
                            required_features: wgpu::Features::PUSH_CONSTANTS,
                            required_limits: adapter.limits(),
                            memory_hints: wgpu::MemoryHints::Performance,
                            trace: wgpu::Trace::Off,
                        }),
                        ..Default::default()
                    },
                ),
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
