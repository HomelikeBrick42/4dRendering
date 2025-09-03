pub mod camera;
pub mod objects;

use crate::{
    camera::Camera,
    objects::{Group, Hyperplane, Hypersphere, Objects},
};
use eframe::{egui, wgpu};
use egui_file_dialog::FileDialog;
use math::Rotor;
use rendering::{RenderData, RenderState, RenderTarget, ViewAxes, register_rendering_state};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::{f32::consts::TAU, sync::Arc, time::Instant};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct UISettings {
    info_window_open: bool,
    camera_window_open: bool,
    xwz_window_open: bool,
    xyw_window_open: bool,
    objects_view: ObjectsView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ObjectsView {
    Flat,
    Grouped,
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            info_window_open: true,
            camera_window_open: true,
            xwz_window_open: true,
            xyw_window_open: true,
            objects_view: ObjectsView::Grouped,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct Scene {
    camera: Camera,
    objects: Objects,
}

impl Default for Scene {
    fn default() -> Self {
        let camera = Camera::new(cgmath::Vector4 {
            x: -3.0,
            y: 1.0,
            z: 0.0,
            w: 0.0,
        });

        let mut objects = Objects {
            groups: SlotMap::with_key(),
            hyperspheres: SlotMap::with_key(),
            hyperplanes: SlotMap::with_key(),
        };

        objects.groups.insert(Group {
            name: "Test Group".into(),
            transform: objects::Transform::default(),
        });
        objects.hyperspheres.insert(Hypersphere {
            name: "Red".into(),
            group: None,
            transform: objects::Transform {
                position: cgmath::Vector4 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                    w: 0.0,
                },
                ..Default::default()
            },
            color: cgmath::Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 1.0,
        });
        objects.hyperplanes.insert(Hyperplane {
            name: "Ground".into(),
            group: None,
            transform: objects::Transform {
                position: cgmath::Vector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0,
                },
                ..Default::default()
            },
            width: 5.0,
            height: 5.0,
            depth: 5.0,
            color: cgmath::Vector3 {
                x: 0.2,
                y: 0.8,
                z: 0.3,
            },
        });

        Self { camera, objects }
    }
}

struct App {
    last_time: Option<Instant>,

    xyz_render_target: RenderTarget,
    xwz_render_target: RenderTarget,
    xyw_render_target: RenderTarget,

    ui_settings: UISettings,
    scene: Scene,

    file_dialog: FileDialog,
    file_interaction: FileInteraction,
}

enum FileInteraction {
    None,
    Save,
    Load,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let eframe::egui_wgpu::RenderState { device, .. } = cc.wgpu_render_state.as_ref().unwrap();

        register_rendering_state(cc);

        Self {
            last_time: None,

            xyz_render_target: RenderTarget::new(device, 1, 1),
            xwz_render_target: RenderTarget::new(device, 1, 1),
            xyw_render_target: RenderTarget::new(device, 1, 1),

            ui_settings: cc
                .storage
                .unwrap()
                .get_string("ui_settings")
                .and_then(|str| serde_json::from_str(&str).ok())
                .unwrap_or_default(),
            scene: cc
                .storage
                .unwrap()
                .get_string("scene")
                .and_then(|str| serde_json::from_str(&str).ok())
                .unwrap_or_default(),

            file_dialog: FileDialog::new()
                .add_file_filter_extensions("Scene", vec!["scene"])
                .default_file_filter("Scene")
                .add_save_extension("Scene", "scene")
                .default_save_extension("Scene"),
            file_interaction: FileInteraction::None,
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
                if ui.button("Load").clicked() {
                    self.file_interaction = FileInteraction::Load;
                    self.file_dialog.pick_file();
                }
                if ui.button("Save").clicked() {
                    self.file_interaction = FileInteraction::Save;
                    self.file_dialog.save_file();
                }
                self.ui_settings.info_window_open |= ui.button("Info").clicked();
                self.ui_settings.camera_window_open |= ui.button("Camera").clicked();
                self.ui_settings.xwz_window_open |= ui.button("XWZ View").clicked();
                self.ui_settings.xyw_window_open |= ui.button("XYW View").clicked();
            });
        });

        egui::SidePanel::left("Objects").show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("View Type:");
                    egui::ComboBox::new("View Type", "")
                        .selected_text(match self.ui_settings.objects_view {
                            ObjectsView::Flat => "Flat",
                            ObjectsView::Grouped => "Grouped",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.ui_settings.objects_view,
                                ObjectsView::Flat,
                                "Flat",
                            );
                            ui.selectable_value(
                                &mut self.ui_settings.objects_view,
                                ObjectsView::Grouped,
                                "Grouped",
                            );
                        });
                });
                match self.ui_settings.objects_view {
                    ObjectsView::Flat => self.scene.objects.flat_ui(ui),
                    ObjectsView::Grouped => self.scene.objects.grouped_ui(ui),
                }
            });
            ui.allocate_space(ui.available_size());
        });

        self.file_dialog.update(ctx);
        if let Some(mut path) = self.file_dialog.take_picked() {
            match std::mem::replace(&mut self.file_interaction, FileInteraction::None) {
                FileInteraction::None => {}
                FileInteraction::Save => {
                    if path.extension().is_none() {
                        path.set_extension("scene");
                    }
                    let state = serde_json::to_string(&self.scene).unwrap();
                    if let Err(e) = std::fs::write(&path, state) {
                        eprintln!("Error when writing scene '{}': {e}", path.to_string_lossy());
                    }
                }
                FileInteraction::Load => {
                    if let Ok(s) = std::fs::read_to_string(&path).inspect_err(|e| {
                        eprintln!("Error when loading scene '{}': {e}", path.to_string_lossy());
                    }) && let Ok(state) = serde_json::from_str(&s).inspect_err(|e| {
                        eprintln!(
                            "Error when deserialising scene '{}': {e}",
                            path.to_string_lossy()
                        );
                    }) {
                        self.scene = state;
                    }
                }
            }
        }

        {
            let mut reset = false;
            egui::Window::new("Info")
                .open(&mut self.ui_settings.info_window_open)
                .scroll(true)
                .show(ctx, |ui| {
                    ui.label(format!("FPS: {:.3}", 1.0 / dt));
                    ui.label(format!("Frame Time: {:.3}ms", 1000.0 * dt));
                    reset |= ui.button("RESET EVERYTHING").clicked();
                    ui.allocate_space(ui.available_size());
                });
            if reset {
                self.ui_settings = Default::default();
                self.scene = Default::default();
            }
        }

        egui::Window::new("Camera")
            .open(&mut self.ui_settings.camera_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui_vector4(ui, &mut self.scene.camera.position);
                });
                ui.horizontal(|ui| {
                    ui.label("Move Speed:");
                    ui.add(egui::DragValue::new(&mut self.scene.camera.move_speed).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Rotation Speed:");
                    ui.add(egui::DragValue::new(&mut self.scene.camera.rotation_speed).speed(0.1));
                    self.scene.camera.rotation_speed = self.scene.camera.rotation_speed.max(0.0);
                });
                ui.collapsing("Align", |ui| {
                    if ui.button("Reset XY Rotation").clicked() {
                        self.scene.camera.xy_rotation = 0.0;
                    }
                    if ui.button("Rotate to WYZ").clicked() {
                        self.scene.camera.main_rotation = self
                            .scene
                            .camera
                            .main_rotation
                            .then(Rotor::rotate_xw(0.25 * TAU));
                    }
                    if ui.button("Rotate to XYW").clicked() {
                        self.scene.camera.main_rotation = self
                            .scene
                            .camera
                            .main_rotation
                            .then(Rotor::rotate_zw(0.25 * TAU));
                    }
                    ui.label("These align buttons assume that the current XY rotation is 0");
                    if ui.button("Align XYZ").clicked() {
                        self.scene.camera.main_rotation = Rotor::identity();
                    }
                    if ui.button("Align WYZ").clicked() {
                        self.scene.camera.main_rotation = Rotor::rotate_xw(0.25 * TAU);
                    }
                    if ui.button("Align XYW").clicked() {
                        self.scene.camera.main_rotation = Rotor::rotate_zw(0.25 * TAU);
                    }
                });
                ui.add_enabled_ui(false, |ui| {
                    let transform = self.scene.camera.transform();
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui_vector4(ui, &mut transform.position());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Forward:");
                        ui_vector4(ui, &mut transform.x());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Up:");
                        ui_vector4(ui, &mut transform.y());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Right:");
                        ui_vector4(ui, &mut transform.z());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Ana:");
                        ui_vector4(ui, &mut transform.w());
                    });
                });
                ui.allocate_space(ui.available_size());
            });

        {
            let callback_resources = &mut renderer.write().callback_resources;
            let render_state: &mut RenderState = callback_resources.get_mut().unwrap();

            render_state.update_hyperspheres(device, queue, self.scene.objects.gpu_hyperspheres());
            render_state.update_hyperplanees(device, queue, self.scene.objects.gpu_hyperplanes());
        }

        if !ctx.wants_keyboard_input() && !ctx.is_using_pointer() {
            ctx.input(|i| self.scene.camera.update(dt, i));
        }

        egui::Window::new("XWZ View")
            .frame(egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::ZERO))
            .open(&mut self.ui_settings.xwz_window_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui_render_target(
                    ui,
                    device,
                    &mut self.xwz_render_target,
                    &self.scene.camera,
                    ViewAxes::XWZ,
                    ui.available_size(),
                );
            });

        egui::Window::new("XYW View")
            .frame(egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::ZERO))
            .open(&mut self.ui_settings.xyw_window_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui_render_target(
                    ui,
                    device,
                    &mut self.xyw_render_target,
                    &self.scene.camera,
                    ViewAxes::XYW,
                    ui.available_size(),
                );
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui_render_target(
                    ui,
                    device,
                    &mut self.xyz_render_target,
                    &self.scene.camera,
                    ViewAxes::XYZ,
                    ui.available_size(),
                );
            });

        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(
            "ui_settings",
            serde_json::to_string(&self.ui_settings).unwrap(),
        );
        storage.set_string("scene", serde_json::to_string(&self.scene).unwrap());
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

fn ui_render_target(
    ui: &mut egui::Ui,
    device: &wgpu::Device,
    render_target: &mut RenderTarget,
    camera: &Camera,
    view_axes: ViewAxes,
    size: egui::Vec2,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::all());

    render_target.maybe_resize(device, rect.width() as _, rect.height() as _);
    ui.painter()
        .add(eframe::egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderData {
                render_target: render_target.clone(),
                camera_transform: camera.transform(),
                view_axes,
            },
        ));

    response
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
