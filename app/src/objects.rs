use slotmap::{SlotMap, new_key_type};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: cgmath::Vector4<f32>,
}

impl Transform {
    pub fn transform(&self) -> math::Transform {
        math::Transform::translation(self.position)
    }
}

#[derive(Debug, Clone)]
pub struct Hypersphere {
    pub name: String,
    pub group: Option<GroupTransformID>,
    pub transform: Transform,
    pub radius: f32,
    pub color: cgmath::Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct Hyperplane {
    pub name: String,
    pub group: Option<GroupTransformID>,
    pub transform: Transform,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub color: cgmath::Vector3<f32>,
}

new_key_type! {
    pub struct GroupTransformID;
    pub struct HypersphereID;
    pub struct HyperplaneID;
}

#[derive(Debug, Clone)]
pub struct Objects {
    pub group_transforms: SlotMap<GroupTransformID, Transform>,
    pub hyperspheres: SlotMap<HypersphereID, Hypersphere>,
    pub hyperplanes: SlotMap<HyperplaneID, Hyperplane>,
}

impl Objects {
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
                transform: if let Some(group_id) = group
                    && let Some(group_transform) = self.group_transforms.get(group_id)
                {
                    group_transform.transform().then(transform.transform())
                } else {
                    transform.transform()
                },
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
                transform: if let Some(group_id) = group
                    && let Some(group_transform) = self.group_transforms.get(group_id)
                {
                    group_transform.transform().then(transform.transform())
                } else {
                    transform.transform()
                },
                color,
                width,
                height,
                depth,
                _padding: Default::default(),
            },
        )
    }
}
