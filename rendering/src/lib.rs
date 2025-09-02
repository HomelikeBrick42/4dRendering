mod render_target;

pub use render_target::RenderTarget;

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
    full_screen_quad_render_pipeline: wgpu::RenderPipeline,
}

pub fn register_rendering_state(cc: &eframe::CreationContext<'_>) {
    let eframe::egui_wgpu::RenderState {
        device,
        renderer,
        target_format,
        ..
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

    let ray_tracing_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/ray_tracing.wgsl"));
    let ray_tracing_compute_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Tracing Compute Pipeline Layout"),
            bind_group_layouts: &[
                &render_target::write_bind_group_layout(device),
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
            module: &ray_tracing_shader,
            entry_point: Some("ray_trace"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

    let full_screen_quad_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/full_screen_quad.wgsl"));
    let full_screen_quad_render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Full Screen Quad Render Pipeline Layout"),
            bind_group_layouts: &[&render_target::sample_bind_group_layout(device)],
            push_constant_ranges: &[],
        });
    let full_screen_quad_render_pipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Full Screen Quad Render Pipeline"),
            layout: Some(&full_screen_quad_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &full_screen_quad_shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &full_screen_quad_shader,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: *target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            multiview: None,
            cache: None,
        });

    renderer.write().callback_resources.insert(RenderState {
        scene_info_buffer,
        scene_info_bind_group,

        hyper_spheres_bind_group_layout,
        hyper_spheres_buffer,
        hyper_spheres_bind_group,

        ray_tracing_compute_pipeline,
        full_screen_quad_render_pipeline,
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
    pub render_target: RenderTarget,
    pub camera_transform: Transform,
    pub view_axes: ViewAxes,
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
            compute_pass.set_bind_group(0, &self.render_target.write_bind_group, &[]);
            compute_pass.set_bind_group(1, &state.scene_info_bind_group, &[]);
            compute_pass.set_bind_group(2, &state.hyper_spheres_bind_group, &[]);

            let camera = {
                let x = self.camera_transform.x();
                let y = self.camera_transform.y();
                let z = self.camera_transform.z();
                let w = self.camera_transform.w();
                let (forward, up, right) = match self.view_axes {
                    ViewAxes::XYZ => (x, y, z),
                    ViewAxes::XWZ => (x, w, z),
                    ViewAxes::XYW => (x, y, w),
                };
                Camera {
                    position: self.camera_transform.position(),
                    forward,
                    up,
                    right,
                }
            };
            compute_pass.set_push_constants(0, bytemuck::bytes_of(&camera));

            let (width, height) = self.render_target.size();
            compute_pass.dispatch_workgroups(width.div_ceil(16), height.div_ceil(16), 1);
        }

        vec![encoder.finish()]
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        let state: &RenderState = callback_resources.get().unwrap();

        render_pass.set_pipeline(&state.full_screen_quad_render_pipeline);
        render_pass.set_bind_group(0, &self.render_target.sample_bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
