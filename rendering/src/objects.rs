use math::Transform;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Hypersphere {
    pub transform: Transform,
    pub color: cgmath::Vector3<f32>,
    pub radius: f32,
}

unsafe impl bytemuck::Zeroable for Hypersphere {}
unsafe impl bytemuck::Pod for Hypersphere {}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Hyperplane {
    pub transform: Transform,
    pub color: cgmath::Vector3<f32>,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub _padding: [f32; 2],
}

unsafe impl bytemuck::Zeroable for Hyperplane {}
unsafe impl bytemuck::Pod for Hyperplane {}
