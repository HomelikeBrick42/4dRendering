use eframe::{egui, wgpu};
use math::Transform;
use std::mem::offset_of;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Camera {
    pub position: cgmath::Vector4<f32>,
    pub forward: cgmath::Vector4<f32>,
    pub up: cgmath::Vector4<f32>,
    pub right: cgmath::Vector4<f32>,
    pub aspect: f32,
}

unsafe impl bytemuck::Zeroable for Camera {}
unsafe impl bytemuck::Pod for Camera {}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct SceneInfo {
    hyper_sphere_count: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct HyperSphere {
    pub position: cgmath::Vector4<f32>,
    pub color: cgmath::Vector3<f32>,
    pub radius: f32,
}

unsafe impl bytemuck::Zeroable for HyperSphere {}
unsafe impl bytemuck::Pod for HyperSphere {}

pub struct RenderState {
    scene_info_buffer: wgpu::Buffer,
    scene_info_bind_group: wgpu::BindGroup,

    hyper_spheres_bind_group_layout: wgpu::BindGroupLayout,
    hyper_spheres_buffer: wgpu::Buffer,
    hyper_spheres_bind_group: wgpu::BindGroup,

    ray_tracing_compute_pipeline: wgpu::ComputePipeline,
}

pub fn register_rendering_state(cc: &eframe::CreationContext<'_>) {
    let eframe::egui_wgpu::RenderState {
        device, renderer, ..
    } = cc.wgpu_render_state.as_ref().unwrap();

    let scene_info_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene Info Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let scene_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Scene Info Buffer"),
        size: size_of::<SceneInfo>().try_into().unwrap(),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let scene_info_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Scene Info Bind Group"),
        layout: &scene_info_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: scene_info_buffer.as_entire_binding(),
        }],
    });

    let hyper_spheres_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hyper Spheres Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let hyper_spheres_buffer = hyper_spheres_buffer(device, 0);
    let hyper_spheres_bind_group = hyper_spheres_bind_group(
        device,
        &hyper_spheres_bind_group_layout,
        &hyper_spheres_buffer,
    );

    let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/ray_tracing.wgsl"));

    let ray_tracing_compute_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Tracing Compute Pipeline Layout"),
            bind_group_layouts: &[
                &scene_info_bind_group_layout,
                &hyper_spheres_bind_group_layout,
            ],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..size_of::<Camera>() as _,
            }],
        });
    let ray_tracing_compute_pipeline =
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Tracing Compute Pipeline"),
            layout: Some(&ray_tracing_compute_pipeline_layout),
            module: &shader,
            entry_point: Some("ray_trace"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

    renderer.write().callback_resources.insert(RenderState {
        scene_info_buffer,
        scene_info_bind_group,

        hyper_spheres_bind_group_layout,
        hyper_spheres_buffer,
        hyper_spheres_bind_group,

        ray_tracing_compute_pipeline,
    });
}

fn hyper_spheres_buffer(device: &wgpu::Device, length: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Hyper Spheres Buffer"),
        size: (length.max(1) * size_of::<HyperSphere>())
            .try_into()
            .unwrap(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn hyper_spheres_bind_group(
    device: &wgpu::Device,
    hyper_spheres_bind_group_layout: &wgpu::BindGroupLayout,
    hyper_spheres_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Hyper Spheres Bind Group"),
        layout: hyper_spheres_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: hyper_spheres_buffer.as_entire_binding(),
        }],
    })
}

impl RenderState {
    pub fn update_hyper_spheres(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        hyper_spheres: &[HyperSphere],
    ) {
        if size_of_val(hyper_spheres) > self.hyper_spheres_buffer.size() as _ {
            self.hyper_spheres_buffer = hyper_spheres_buffer(device, hyper_spheres.len());
            self.hyper_spheres_bind_group = hyper_spheres_bind_group(
                device,
                &self.hyper_spheres_bind_group_layout,
                &self.hyper_spheres_buffer,
            );
        }
        queue.write_buffer(
            &self.hyper_spheres_buffer,
            0,
            bytemuck::cast_slice(hyper_spheres),
        );
        queue.write_buffer(
            &self.scene_info_buffer,
            offset_of!(SceneInfo, hyper_sphere_count) as _,
            &u32::to_ne_bytes(hyper_spheres.len().try_into().unwrap()),
        );
        queue.submit(std::iter::empty());
    }
}

pub enum ViewAxes {
    XYZ,
    XWZ,
    XYW,
}

pub struct RenderData {
    pub camera_transform: Transform,
    pub view_axes: ViewAxes,
    pub aspect: f32,
}

impl eframe::egui_wgpu::CallbackTrait for RenderData {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let state: &mut RenderState = callback_resources.get_mut().unwrap();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Ray Tracing Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracing Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&state.ray_tracing_compute_pipeline);
            compute_pass.set_bind_group(0, &state.scene_info_bind_group, &[]);
            compute_pass.set_bind_group(1, &state.hyper_spheres_bind_group, &[]);
            compute_pass.set_push_constants(
                0,
                bytemuck::bytes_of(&{
                    let (forward, up, right) = match self.view_axes {
                        ViewAxes::XYZ => (
                            self.camera_transform.x(),
                            self.camera_transform.y(),
                            self.camera_transform.z(),
                        ),
                        ViewAxes::XWZ => (
                            self.camera_transform.x(),
                            self.camera_transform.w(),
                            self.camera_transform.z(),
                        ),
                        ViewAxes::XYW => (
                            self.camera_transform.x(),
                            self.camera_transform.y(),
                            self.camera_transform.w(),
                        ),
                    };
                    Camera {
                        position: self.camera_transform.position(),
                        forward,
                        up,
                        right,
                        aspect: self.aspect,
                    }
                }),
            );
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        vec![encoder.finish()]
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        _render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        let _state: &RenderState = callback_resources.get().unwrap();
    }
}
