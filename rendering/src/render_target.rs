use eframe::wgpu;

#[derive(Debug, Clone)]
pub struct RenderTarget {
    write_bind_group_layout: wgpu::BindGroupLayout,
    sample_bind_group_layout: wgpu::BindGroupLayout,

    texture: wgpu::Texture,

    pub(crate) write_bind_group: wgpu::BindGroup,
    pub(crate) sample_bind_group: wgpu::BindGroup,
}

impl RenderTarget {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);

        let write_bind_group_layout = write_bind_group_layout(device);
        let sample_bind_group_layout = sample_bind_group_layout(device);

        let texture = texture(
            device,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&Default::default());
        let write_bind_group = write_bind_group(device, &write_bind_group_layout, &texture_view);
        let sample_bind_group = sample_bind_group(device, &sample_bind_group_layout, &texture_view);

        Self {
            write_bind_group_layout,
            sample_bind_group_layout,

            texture,

            write_bind_group,
            sample_bind_group,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        let wgpu::Extent3d { width, height, .. } = self.texture.size();
        (width, height)
    }

    pub fn maybe_resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);

        let new_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        if new_size != self.texture.size() {
            self.texture = texture(device, new_size);

            let texture_view = self.texture.create_view(&Default::default());
            self.write_bind_group =
                write_bind_group(device, &self.write_bind_group_layout, &texture_view);
            self.sample_bind_group =
                sample_bind_group(device, &self.sample_bind_group_layout, &texture_view);
        }
    }
}

fn texture(device: &wgpu::Device, size: wgpu::Extent3d) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("RenderTarget Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

pub(crate) fn write_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Write RenderTarget Texture Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba32Float,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }],
    })
}

fn write_bind_group(
    device: &wgpu::Device,
    write_bind_group_layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Write RenderTarget Texture Bind Group"),
        layout: write_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(texture_view),
        }],
    })
}

pub(crate) fn sample_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Sample RenderTarget Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    })
}

fn sample_bind_group(
    device: &wgpu::Device,
    sample_bind_group_layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("RenderTarget Texture Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Sample RenderTarget Texture Bind Group"),
        layout: sample_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}
