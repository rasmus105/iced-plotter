//! GPU rendering pipeline for the plotter.

use crate::gpu_types::{RawPoint, Uniforms};
use iced::wgpu;

/// A dynamically resizable GPU buffer.
pub struct DynamicBuffer {
    pub buffer: wgpu::Buffer,
    capacity: u64,
    usage: wgpu::BufferUsages,
    label: &'static str,
}

impl DynamicBuffer {
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        initial_capacity: u64,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: initial_capacity,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity: initial_capacity,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            label,
        }
    }

    /// Ensure the buffer can hold at least `size` bytes, recreating if needed.
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, size: u64) {
        if size > self.capacity {
            // Grow by 50% or to required size, whichever is larger
            let new_capacity = (self.capacity * 3 / 2).max(size);
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: new_capacity,
                usage: self.usage,
                mapped_at_creation: false,
            });
            self.capacity = new_capacity;
        }
    }
}

/// The GPU rendering pipeline for the plotter.
pub struct Pipeline {
    marker_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    point_buffer: DynamicBuffer,
    line_buffer: DynamicBuffer,
    grid_buffer: DynamicBuffer,
    uniform_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("plot_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shaders/plot.wgsl"
            ))),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("plot_uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("plot_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("plot_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("plot_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Point vertex buffer layout (per-instance data)
        let point_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RawPoint>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 24,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 28,
                    shader_location: 3,
                },
            ],
        };

        // Line vertex buffer layout - uses RawPoint but only reads position and color
        let line_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RawPoint>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1,
                },
                // Lines don't use distance/pattern yet - will add later if needed
            ],
        };

        // Blend state for transparency
        let blend_state = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };

        // Create marker pipeline
        let marker_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("marker_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_marker"),
                buffers: &[point_vertex_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_marker"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        // Create line pipeline
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("line_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_line"),
                buffers: &[line_vertex_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_line"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        // Create dynamic buffers
        let point_buffer = DynamicBuffer::new(
            device,
            "point_buffer",
            1024 * std::mem::size_of::<RawPoint>() as u64,
            wgpu::BufferUsages::VERTEX,
        );

        let line_buffer = DynamicBuffer::new(
            device,
            "line_buffer",
            1024 * std::mem::size_of::<RawPoint>() as u64,
            wgpu::BufferUsages::VERTEX,
        );

        let grid_buffer = DynamicBuffer::new(
            device,
            "grid_buffer",
            1024 * std::mem::size_of::<RawPoint>() as u64,
            wgpu::BufferUsages::VERTEX,
        );

        Self {
            marker_pipeline,
            line_pipeline,
            point_buffer,
            line_buffer,
            grid_buffer,
            uniform_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Update GPU buffers with new data.
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uniforms: &Uniforms,
        points: &[RawPoint],
        line_vertices: &[RawPoint],
        grid_vertices: &[RawPoint],
    ) {
        // Update uniforms
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));

        // Update point buffer
        if !points.is_empty() {
            let point_data = bytemuck::cast_slice(points);
            self.point_buffer
                .ensure_capacity(device, point_data.len() as u64);
            queue.write_buffer(&self.point_buffer.buffer, 0, point_data);
        }

        // Update line buffer
        if !line_vertices.is_empty() {
            let line_data = bytemuck::cast_slice(line_vertices);
            self.line_buffer
                .ensure_capacity(device, line_data.len() as u64);
            queue.write_buffer(&self.line_buffer.buffer, 0, line_data);
        }

        if !grid_vertices.is_empty() {
            let grid_data = bytemuck::cast_slice(grid_vertices);
            self.grid_buffer
                .ensure_capacity(device, grid_data.len() as u64);
            queue.write_buffer(&self.grid_buffer.buffer, 0, grid_data);
        }
    }

    /// Render markers (points).
    pub fn render_markers(&self, render_pass: &mut wgpu::RenderPass<'_>, num_points: u32) {
        if num_points == 0 {
            return;
        }

        render_pass.set_pipeline(&self.marker_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.point_buffer.buffer.slice(..));
        // 6 vertices per quad (2 triangles), one instance per point
        render_pass.draw(0..6, 0..num_points);
    }

    /// Render lines.
    pub fn render_lines(&self, render_pass: &mut wgpu::RenderPass<'_>, num_vertices: u32) {
        if num_vertices == 0 {
            return;
        }

        render_pass.set_pipeline(&self.line_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.line_buffer.buffer.slice(..));
        render_pass.draw(0..num_vertices, 0..1);
    }

    pub fn render_grid(&self, render_pass: &mut wgpu::RenderPass<'_>, num_vertices: u32) {
        if num_vertices == 0 {
            return;
        }

        render_pass.set_pipeline(&self.line_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.grid_buffer.buffer.slice(..));
        render_pass.draw(0..num_vertices, 0..1);
    }
}
