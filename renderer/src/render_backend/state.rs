use std::time::Duration;
use cgmath::{InnerSpace, Rotation3, Vector3, Zero};
use wgpu::include_wgsl;
use wgpu::util::{DeviceExt};
use crate::render_backend::{context::Context, instance::Instance, vertex::Vertex};
use crate::render_backend::camera::{Camera, CameraController, CameraUniform, Projection};
use crate::render_backend::instance::InstanceRaw;
use crate::render_backend::texture::Texture;

pub struct State {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    veritces : Vec<Vertex>,
    index_buffer: wgpu::Buffer,
    projection: Projection,
    depth_texture: Texture,
    pub instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera: Camera,
    pub camera_controller: CameraController,
    camera_bind_group: wgpu::BindGroup,
    context: Context,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_vertex_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

const VERTICES: &[Vertex] = &[
    // Front face (+Z) - normal pointant vers +Z
    Vertex { position: [-0.5, -0.5,  0.5] },
    Vertex { position: [ 0.5, -0.5,  0.5] },
    Vertex { position: [ 0.5,  0.5,  0.5] },
    Vertex { position: [-0.5,  0.5,  0.5] },

    // Back face (-Z) - normal pointant vers -Z
    Vertex { position: [ 0.5, -0.5, -0.5] },
    Vertex { position: [-0.5, -0.5, -0.5] },
    Vertex { position: [-0.5,  0.5, -0.5] },
    Vertex { position: [ 0.5,  0.5, -0.5] },

    // Left face (-X) - normal pointant vers -X
    Vertex { position: [-0.5, -0.5, -0.5] },
    Vertex { position: [-0.5, -0.5,  0.5] },
    Vertex { position: [-0.5,  0.5,  0.5] },
    Vertex { position: [-0.5,  0.5, -0.5] },

    // Right face (+X) - normal pointant vers +X
    Vertex { position: [ 0.5, -0.5,  0.5] },
    Vertex { position: [ 0.5, -0.5, -0.5] },
    Vertex { position: [ 0.5,  0.5, -0.5] },
    Vertex { position: [ 0.5,  0.5,  0.5] },

    // Top face (+Y) - normal pointant vers +Y
    Vertex { position: [-0.5,  0.5,  0.5] },
    Vertex { position: [ 0.5,  0.5,  0.5] },
    Vertex { position: [ 0.5,  0.5, -0.5] },
    Vertex { position: [-0.5,  0.5, -0.5] },

    // Bottom face (-Y) - normal pointant vers -Y
    Vertex { position: [-0.5, -0.5, -0.5] },
    Vertex { position: [ 0.5, -0.5, -0.5] },
    Vertex { position: [ 0.5, -0.5,  0.5] },
    Vertex { position: [-0.5, -0.5,  0.5] },
];

const INDICES: &[u16] = &[
    0,  1,  2,  2,  3,  0, // Front
    4,  5,  6,  6,  7,  4, // Back
    8,  9, 10, 10, 11,  8, // Left
    12, 13, 14, 14, 15, 12, // Right
    16, 17, 18, 18, 19, 16, // Top
    20, 21, 22, 22, 23, 20, // Bottom
];

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

impl State {
    pub async fn new(context: Context) -> Self {
        let shader = context.device.create_shader_module(include_wgsl!("../shaders/shader.wgsl").into());

        // Camera setup
        let camera = Camera::new(
            (0.0, 5.0, 10.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(-20.0),
        );
        let projection = Projection::new(
            context.config.width,
            context.config.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
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

        let camera_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let depth_texture = Texture::create_depth_texture(&context, "depth_texture");

        let vertex_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light_vertex_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let instances = (0..1).flat_map(|z| {
            (0..1).map(move |x| {
                let position = Vector3 { x: x as f32 * 2.0, y: 0.0, z: z as f32 * 2.0 } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can affect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
                };

                Instance { position, rotation }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light_uniform = LightUniform {
            position: [0.0, 1.0, 0.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: None,
            });

        let light_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let render_pipeline_layout =
            context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                immediate_size: 0,
            });

        let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview_mask: None,
        });

        let layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
            immediate_size: 0,
        });

        let light_shader = context.device.create_shader_module(include_wgsl!("../shaders/light.wgsl").into());

        let light_render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Light Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &light_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &light_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.config.format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(Texture::DEPTH_FORMAT).map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // tells wgpu to render to just specific texture layers.
            multiview_mask: None,
            cache: None,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            veritces: Vec::from(VERTICES),
            index_buffer,
            projection,
            depth_texture,
            instances,
            instance_buffer,
            camera_uniform,
            camera_buffer,
            camera,
            camera_controller: CameraController::new(4.0, 0.4),
            camera_bind_group,
            context,
            light_uniform,
            light_buffer,
            light_vertex_buffer,
            light_bind_group,
            light_render_pipeline,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.resize(width, height);
        if width > 0 && height > 0 {
            self.context.config.width = width;
            self.context.config.height = height;
            self.context.surface.configure(&self.context.device, &self.context.config);
            self.depth_texture = Texture::create_depth_texture(&self.context, "depth_texture");
        }
        }

    pub fn update(&mut self, delta_time: Duration) {
        self.camera_controller.update_camera(&mut self.camera, delta_time);
        self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        self.light_uniform.position[0] += 3.0 * delta_time.as_secs_f32();
        let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        self.context.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
        self.context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.context.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }

    pub fn update_vert(&mut self, new_vert: Vec<Vertex>) {
        self.vertex_buffer = self.context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&new_vert),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        self.veritces = new_vert;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.context.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.light_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..self.instances.len() as _);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.draw(0..self.veritces.len() as u32, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}