use math::Transform;
use slotmap::{SlotMap, new_key_type};

#[derive(Debug)]
pub struct Hypersphere {
    pub name: String,
    pub position: cgmath::Vector4<f32>,
    pub radius: f32,
    pub color: cgmath::Vector3<f32>,
}

impl Hypersphere {
    pub fn to_gpu_hypersphere(&self) -> rendering::objects::Hypersphere {
        let Self {
            name: _,
            position,
            radius,
            color,
        } = *self;
        rendering::objects::Hypersphere {
            position,
            color,
            radius,
        }
    }
}

#[derive(Debug)]
pub struct Hyperplane {
    pub name: String,
    pub position: cgmath::Vector4<f32>,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub color: cgmath::Vector3<f32>,
}

impl Hyperplane {
    pub fn transform(&self) -> Transform {
        Transform::translation(self.position)
    }

    pub fn to_gpu_hyperplane(&self) -> rendering::objects::Hyperplane {
        let Self {
            name: _,
            position: _,
            width,
            height,
            depth,
            color,
        } = *self;
        rendering::objects::Hyperplane {
            transform: self.transform(),
            color,
            width,
            height,
            depth,
            _padding: Default::default(),
        }
    }
}

new_key_type! {
    pub struct HypersphereID;
    pub struct HyperplaneID;
}

pub struct Objects {
    pub hyperspheres: SlotMap<HypersphereID, Hypersphere>,
    pub hyperplanes: SlotMap<HyperplaneID, Hyperplane>,
}
