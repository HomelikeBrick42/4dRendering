use crate::ui_vector4;
use eframe::egui;
use math::Rotor;
use serde::{Deserialize, Serialize};
use slotmap::{SlotMap, new_key_type};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: cgmath::Vector4<f32>,
    pub xy_rotation: f32,
    pub xz_rotation: f32,
    pub xw_rotation: f32,
    pub yz_rotation: f32,
    pub yw_rotation: f32,
    pub zw_rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: cgmath::Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            },
            xy_rotation: 0.0,
            xz_rotation: 0.0,
            xw_rotation: 0.0,
            yz_rotation: 0.0,
            yw_rotation: 0.0,
            zw_rotation: 0.0,
        }
    }
}

impl Transform {
    pub fn transform(&self) -> math::Transform {
        math::Transform::translation(self.position).then(math::Transform::from_rotor(
            Rotor::rotate_xy(self.xy_rotation)
                .then(Rotor::rotate_xz(self.xz_rotation))
                .then(Rotor::rotate_xw(self.xw_rotation))
                .then(Rotor::rotate_yz(self.yz_rotation))
                .then(Rotor::rotate_yw(self.yw_rotation))
                .then(Rotor::rotate_zw(self.zw_rotation)),
        ))
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Position:");
            ui_vector4(ui, &mut self.position);
        });
        ui.horizontal(|ui| {
            ui.label("XY Rotation:");
            ui.drag_angle(&mut self.xy_rotation);
        });
        ui.horizontal(|ui| {
            ui.label("XZ Rotation:");
            ui.drag_angle(&mut self.xz_rotation);
        });
        ui.horizontal(|ui| {
            ui.label("XW Rotation:");
            ui.drag_angle(&mut self.xw_rotation);
        });
        ui.horizontal(|ui| {
            ui.label("YZ Rotation:");
            ui.drag_angle(&mut self.yz_rotation);
        });
        ui.horizontal(|ui| {
            ui.label("YW Rotation:");
            ui.drag_angle(&mut self.yw_rotation);
        });
        ui.horizontal(|ui| {
            ui.label("ZW Rotation:");
            ui.drag_angle(&mut self.zw_rotation);
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Group {
    pub name: String,
    pub transform: Transform,
}

impl Default for Group {
    fn default() -> Self {
        Self {
            name: "Default Group".into(),
            transform: Transform::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Hypersphere {
    pub name: String,
    pub group: Option<GroupID>,
    pub transform: Transform,
    pub radius: f32,
    pub color: cgmath::Vector3<f32>,
}

impl Default for Hypersphere {
    fn default() -> Self {
        Self {
            name: "Default Hypersphere".into(),
            group: None,
            transform: Transform::default(),
            radius: 1.0,
            color: cgmath::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Hyperplane {
    pub name: String,
    pub group: Option<GroupID>,
    pub transform: Transform,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub color: cgmath::Vector3<f32>,
}

impl Default for Hyperplane {
    fn default() -> Self {
        Self {
            name: "Default Hyperplane".into(),
            group: None,
            transform: Transform::default(),
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            color: cgmath::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
}

new_key_type! {
    pub struct GroupID;
    pub struct HypersphereID;
    pub struct HyperplaneID;
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Objects {
    pub groups: SlotMap<GroupID, Group>,
    pub hyperspheres: SlotMap<HypersphereID, Hypersphere>,
    pub hyperplanes: SlotMap<HyperplaneID, Hyperplane>,
}

impl Objects {
    pub fn cleanup_invalid_ids(&mut self) {
        for hypersphere in self.hyperspheres.values_mut() {
            if let Some(group) = hypersphere.group
                && !self.groups.contains_key(group)
            {
                hypersphere.group = None;
            }
        }
        for hyperplane in self.hyperplanes.values_mut() {
            if let Some(group) = hyperplane.group
                && !self.groups.contains_key(group)
            {
                hyperplane.group = None;
            }
        }
    }

    pub fn flat_ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Groups", |ui| {
            let mut new_id = None;
            if ui.button("New Group").clicked() {
                new_id = Some(self.groups.insert(Group::default()));
            }
            let mut to_delete = vec![];
            for (id, group) in &mut self.groups {
                let response =
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
                            if ui.button("Delete").clicked() {
                                to_delete.push(id);
                            }
                        });
                if new_id == Some(id) {
                    ui.scroll_to_rect(response.header_response.rect, Some(egui::Align::TOP));
                }
            }
            for id in to_delete {
                self.groups.remove(id);
            }
        });
        ui.collapsing("Hyperspheres", |ui| {
            let mut new_id = None;
            if ui.button("New Hypersphere").clicked() {
                new_id = Some(self.hyperspheres.insert(Hypersphere::default()));
            }
            let mut to_delete = vec![];
            let ids = self.hyperspheres.keys().collect::<Vec<_>>();
            Self::hyperspheres_ui(
                ui,
                &self.groups,
                &mut self.hyperspheres,
                ids.into_iter(),
                new_id,
                &mut to_delete,
            );
            for id in to_delete {
                self.hyperspheres.remove(id);
            }
        });
        ui.collapsing("Hyperplanes", |ui| {
            let mut new_id = None;
            if ui.button("New Hyperplane").clicked() {
                new_id = Some(self.hyperplanes.insert(Hyperplane::default()));
            }
            let mut to_delete = vec![];
            let ids = self.hyperplanes.keys().collect::<Vec<_>>();
            Self::hyperplanes_ui(
                ui,
                &self.groups,
                &mut self.hyperplanes,
                ids.into_iter(),
                new_id,
                &mut to_delete,
            );
            for id in to_delete {
                self.hyperplanes.remove(id);
            }
        });
        self.cleanup_invalid_ids();
    }

    pub fn grouped_ui(&mut self, ui: &mut egui::Ui) {
        let mut new_group_id = None;
        if ui.button("New Group").clicked() {
            new_group_id = Some(self.groups.insert(Group::default()));
        }
        let mut groups_to_delete = vec![];

        let mut new_hypersphere_id = None;
        if ui.button("New Hypersphere").clicked() {
            new_hypersphere_id = Some(self.hyperspheres.insert(Hypersphere::default()));
        }
        let mut hyperspheres_to_delete = vec![];

        let mut new_hyperplane_id = None;
        if ui.button("New Hyperplane").clicked() {
            new_hyperplane_id = Some(self.hyperplanes.insert(Hyperplane::default()));
        }
        let mut hyperplanes_to_delete = vec![];

        #[derive(Default)]
        struct GroupedObjects {
            hyperspheres: Vec<HypersphereID>,
            hyperplanes: Vec<HyperplaneID>,
        }
        let mut grouped_objects = BTreeMap::<Option<GroupID>, GroupedObjects>::new();
        for id in self.groups.keys() {
            grouped_objects.entry(Some(id)).or_default();
        }
        for (id, hypersphere) in &self.hyperspheres {
            grouped_objects
                .entry(hypersphere.group)
                .or_default()
                .hyperspheres
                .push(id);
        }
        for (id, hyperplane) in &self.hyperplanes {
            grouped_objects
                .entry(hyperplane.group)
                .or_default()
                .hyperplanes
                .push(id);
        }

        for (id, grouped_objects) in grouped_objects {
            let response = egui::CollapsingHeader::new(if let Some(group_id) = id {
                if let Some(group) = self.groups.get(group_id) {
                    &group.name
                } else {
                    "Invalid"
                }
            } else {
                "None"
            })
            .id_salt(id)
            .show(ui, |ui| {
                if let Some(group_id) = id
                    && let Some(group) = self.groups.get_mut(group_id)
                {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut group.name);
                    });
                    ui.collapsing("Transform", |ui| {
                        group.transform.ui(ui);
                    });
                    if ui.button("Delete").clicked() {
                        groups_to_delete.push(group_id);
                    }
                }
                ui.collapsing("Hyperspheres", |ui| {
                    Self::hyperspheres_ui(
                        ui,
                        &self.groups,
                        &mut self.hyperspheres,
                        grouped_objects.hyperspheres.iter().copied(),
                        new_hypersphere_id,
                        &mut hyperspheres_to_delete,
                    );
                });
                ui.collapsing("Hyperplanes", |ui| {
                    Self::hyperplanes_ui(
                        ui,
                        &self.groups,
                        &mut self.hyperplanes,
                        grouped_objects.hyperplanes.iter().copied(),
                        new_hyperplane_id,
                        &mut hyperplanes_to_delete,
                    );
                });
            });

            if let Some(id) = id
                && new_group_id == Some(id)
            {
                ui.scroll_to_rect(response.header_response.rect, Some(egui::Align::TOP));
            }
        }

        for id in groups_to_delete {
            self.groups.remove(id);
        }
        for id in hyperspheres_to_delete {
            self.hyperspheres.remove(id);
        }
        for id in hyperplanes_to_delete {
            self.hyperplanes.remove(id);
        }

        self.cleanup_invalid_ids();
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

    fn hyperspheres_ui(
        ui: &mut egui::Ui,
        groups: &SlotMap<GroupID, Group>,
        hyperspheres: &mut SlotMap<HypersphereID, Hypersphere>,
        hypersphere_ids: impl Iterator<Item = HypersphereID>,
        scroll_to_id: Option<HypersphereID>,
        to_delete: &mut Vec<HypersphereID>,
    ) {
        for id in hypersphere_ids {
            let hypersphere = &mut hyperspheres[id];
            let response = egui::CollapsingHeader::new(
                egui::RichText::new(&hypersphere.name).color(color_to_egui(hypersphere.color)),
            )
            .id_salt(id)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut hypersphere.name);
                });
                Self::group_ui(ui, groups, &mut hypersphere.group);
                Self::transform_ui(ui, groups, &mut hypersphere.transform, hypersphere.group);
                ui.horizontal(|ui| {
                    ui.label("Radius:");
                    ui.add(egui::DragValue::new(&mut hypersphere.radius).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_rgb(hypersphere.color.as_mut());
                });
                if ui.button("Delete").clicked() {
                    to_delete.push(id);
                }
            });
            if scroll_to_id == Some(id) {
                ui.scroll_to_rect(response.header_response.rect, Some(egui::Align::TOP));
            }
        }
    }

    fn hyperplanes_ui(
        ui: &mut egui::Ui,
        groups: &SlotMap<GroupID, Group>,
        hyperplanes: &mut SlotMap<HyperplaneID, Hyperplane>,
        hyperplane_ids: impl Iterator<Item = HyperplaneID>,
        scroll_to_id: Option<HyperplaneID>,
        to_delete: &mut Vec<HyperplaneID>,
    ) {
        for id in hyperplane_ids {
            let hyperplane = &mut hyperplanes[id];
            let response = egui::CollapsingHeader::new(
                egui::RichText::new(&hyperplane.name).color(color_to_egui(hyperplane.color)),
            )
            .id_salt(id)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut hyperplane.name);
                });
                Self::group_ui(ui, groups, &mut hyperplane.group);
                Self::transform_ui(ui, groups, &mut hyperplane.transform, hyperplane.group);
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
                if ui.button("Delete").clicked() {
                    to_delete.push(id);
                }
            });
            if scroll_to_id == Some(id) {
                ui.scroll_to_rect(response.header_response.rect, Some(egui::Align::TOP));
            }
        }
    }

    fn group_ui(
        ui: &mut egui::Ui,
        groups: &SlotMap<GroupID, Group>,
        group_id: &mut Option<GroupID>,
    ) {
        ui.horizontal(|ui| {
            ui.label("Group:");
            egui::ComboBox::new("Group", "")
                .selected_text(if let Some(group_id) = *group_id {
                    if let Some(group) = groups.get(group_id) {
                        &group.name
                    } else {
                        "Invalid"
                    }
                } else {
                    "None"
                })
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
