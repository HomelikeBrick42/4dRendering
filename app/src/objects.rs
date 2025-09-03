use crate::ui_vector4;
use eframe::egui;
use slotmap::{SlotMap, new_key_type};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: cgmath::Vector4<f32>,
}

impl Transform {
    pub fn transform(&self) -> math::Transform {
        math::Transform::translation(self.position)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Position:");
            ui_vector4(ui, &mut self.position);
        });
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub transform: Transform,
}

#[derive(Debug, Clone)]
pub struct Hypersphere {
    pub name: String,
    pub group: Option<GroupID>,
    pub transform: Transform,
    pub radius: f32,
    pub color: cgmath::Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct Hyperplane {
    pub name: String,
    pub group: Option<GroupID>,
    pub transform: Transform,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub color: cgmath::Vector3<f32>,
}

new_key_type! {
    pub struct GroupID;
    pub struct HypersphereID;
    pub struct HyperplaneID;
}

#[derive(Debug, Clone)]
pub struct Objects {
    pub groups: SlotMap<GroupID, Group>,
    pub hyperspheres: SlotMap<HypersphereID, Hypersphere>,
    pub hyperplanes: SlotMap<HyperplaneID, Hyperplane>,
}

impl Objects {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Groups", |ui| {
            for (id, group) in &mut self.groups {
                egui::CollapsingHeader::new(&group.name)
                    .id_salt(id)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut group.name);
                        });
                        ui.collapsing("Transform", |ui| {
                            group.transform.ui(ui);
                        });
                    });
            }
        });
        ui.collapsing("Hyperspheres", |ui| {
            for (id, hypersphere) in &mut self.hyperspheres {
                egui::CollapsingHeader::new(
                    egui::RichText::new(&hypersphere.name).color(color_to_egui(hypersphere.color)),
                )
                .id_salt(id)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut hypersphere.name);
                    });
                    Self::group_ui(ui, &self.groups, &mut hypersphere.group);
                    Self::transform_ui(
                        ui,
                        &self.groups,
                        &mut hypersphere.transform,
                        hypersphere.group,
                    );
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(&mut hypersphere.radius).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        ui.color_edit_button_rgb(hypersphere.color.as_mut());
                    });
                });
            }
        });
        ui.collapsing("Hyperplanes", |ui| {
            for (id, hyperplane) in &mut self.hyperplanes {
                egui::CollapsingHeader::new(
                    egui::RichText::new(&hyperplane.name).color(color_to_egui(hyperplane.color)),
                )
                .id_salt(id)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut hyperplane.name);
                    });
                    Self::group_ui(ui, &self.groups, &mut hyperplane.group);
                    Self::transform_ui(
                        ui,
                        &self.groups,
                        &mut hyperplane.transform,
                        hyperplane.group,
                    );
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add(egui::DragValue::new(&mut hyperplane.width).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.add(egui::DragValue::new(&mut hyperplane.height).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Depth:");
                        ui.add(egui::DragValue::new(&mut hyperplane.depth).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        ui.color_edit_button_rgb(hyperplane.color.as_mut());
                    });
                });
            }
        });
    }

    pub fn gpu_hyperspheres(
        &self,
    ) -> impl ExactSizeIterator<Item = rendering::objects::Hypersphere> {
        self.hyperspheres.values().map(
            |&Hypersphere {
                 name: _,
                 group,
                 ref transform,
                 radius,
                 color,
             }| rendering::objects::Hypersphere {
                transform: Self::global_transform(&self.groups, transform, group),
                color,
                radius,
            },
        )
    }

    pub fn gpu_hyperplanes(&self) -> impl ExactSizeIterator<Item = rendering::objects::Hyperplane> {
        self.hyperplanes.values().map(
            |&Hyperplane {
                 name: _,
                 group,
                 ref transform,
                 width,
                 height,
                 depth,
                 color,
             }| rendering::objects::Hyperplane {
                transform: Self::global_transform(&self.groups, transform, group),
                color,
                width,
                height,
                depth,
                _padding: Default::default(),
            },
        )
    }

    fn group_ui(
        ui: &mut egui::Ui,
        groups: &SlotMap<GroupID, Group>,
        group_id: &mut Option<GroupID>,
    ) {
        ui.horizontal(|ui| {
            ui.label("Group:");
            egui::ComboBox::new("Group", "")
                .selected_text(
                    if let Some(group_id) = *group_id
                        && let Some(group) = groups.get(group_id)
                    {
                        &group.name
                    } else {
                        "None"
                    },
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(group_id, None, "None");
                    for (id, group) in groups {
                        ui.selectable_value(group_id, Some(id), &group.name);
                    }
                });
        });
    }

    fn transform_ui(
        ui: &mut egui::Ui,
        groups: &SlotMap<GroupID, Group>,
        transform: &mut Transform,
        group: Option<GroupID>,
    ) {
        ui.collapsing("Transform", |ui| {
            transform.ui(ui);
            ui.add_enabled_ui(false, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Global Position:");
                    ui_vector4(
                        ui,
                        &mut Self::global_transform(groups, transform, group).position(),
                    );
                });
            });
        });
    }

    fn global_transform(
        groups: &SlotMap<GroupID, Group>,
        transform: &Transform,
        group: Option<GroupID>,
    ) -> math::Transform {
        if let Some(group_id) = group
            && let Some(group) = groups.get(group_id)
        {
            group.transform.transform().then(transform.transform())
        } else {
            transform.transform()
        }
    }
}

fn color_to_egui(color: cgmath::Vector3<f32>) -> egui::Color32 {
    egui::Color32::from_rgb(
        (color.x.clamp(0.0, 1.0) * 255.0) as u8,
        (color.y.clamp(0.0, 1.0) * 255.0) as u8,
        (color.z.clamp(0.0, 1.0) * 255.0) as u8,
    )
}
