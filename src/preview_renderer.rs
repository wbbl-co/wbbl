use crate::{
    builtin_geometry::{get_cube, get_uv_sphere, BuiltInGeometry},
    compiler_constants::{
        FRAME_BINDING, FRAME_GROUP, GEOMETRY_GROUP, MODEL_TRANSFORM_BINDING, VERTICES_BINDING,
    },
    gltf_encoder,
    model_scene_file_abstractions::EncodedSceneFile,
    shader_layouts::{self, frame::Frame},
};
use std::rc::Rc;

use glam::{Mat3, Mat4, Vec4};
use std::{
    borrow::Cow, collections::HashMap, error::Error, fmt::Display, mem::size_of, num::NonZeroU64,
};
use web_sys::OffscreenCanvas;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, BufferBinding, BufferBindingType, BufferUsages, CompositeAlphaMode, ShaderStages,
    SurfaceTarget,
};

pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4 {
    x_axis: Vec4 {
        x: -1.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    },
    y_axis: Vec4 {
        x: 0.0,
        y: -1.0,
        z: 0.0,
        w: 0.0,
    },
    z_axis: Vec4 {
        x: 0.0,
        y: 0.0,
        z: -1.0,
        w: 0.0,
    },
    w_axis: Vec4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    },
};

#[derive(Debug)]
pub enum PreviewRendererError {
    GeometryTypeNotFound,
}

impl Display for PreviewRendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreviewRendererError::GeometryTypeNotFound => f.write_str("Unexpected Geometry Type"),
        }
    }
}
impl Error for PreviewRendererError {}

pub struct SharedPreviewRendererResources {
    pub device: Rc<wgpu::Device>,
    pub instance: Rc<wgpu::Instance>,
    pub queue: Rc<wgpu::Queue>,
    pub adapter: Rc<wgpu::Adapter>,
    pub geometry: HashMap<BuiltInGeometry, Rc<EncodedSceneFile>>,
    pub geometry_buffers: HashMap<BuiltInGeometry, Rc<wgpu::Buffer>>,
    pub vertices_layout: Rc<wgpu::BindGroupLayout>,
    pub frame_data_layout: Rc<wgpu::BindGroupLayout>,
}

impl SharedPreviewRendererResources {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await?;

        let vertices_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("vertices_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: VERTICES_BINDING,
                visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let frame_data_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame_data_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: FRAME_BINDING,
                    visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: MODEL_TRANSFORM_BINDING,
                    visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let mut cube = get_cube();
        let encoded_cube: Rc<EncodedSceneFile> = gltf_encoder::encode(&mut cube).unwrap().into();

        let mut uv_sphere = get_uv_sphere();
        let encoded_uv_sphere: Rc<EncodedSceneFile> =
            gltf_encoder::encode(&mut uv_sphere).unwrap().into();

        let device: Rc<wgpu::Device> = device.into();

        Ok(SharedPreviewRendererResources {
            device: device.clone(),
            instance: instance.into(),
            queue: queue.into(),
            adapter: adapter.into(),
            geometry: HashMap::from([
                (BuiltInGeometry::Cube, encoded_cube.clone()),
                (BuiltInGeometry::UVSphere, encoded_uv_sphere.clone()),
            ]),
            geometry_buffers: HashMap::from([
                (
                    BuiltInGeometry::Cube,
                    device
                        .clone()
                        .create_buffer_init(&BufferInitDescriptor {
                            label: Some("geometry_buffer"),
                            contents: encoded_cube.clone().buffer.as_slice(),
                            usage: BufferUsages::INDEX | BufferUsages::STORAGE,
                        })
                        .into(),
                ),
                (
                    BuiltInGeometry::UVSphere,
                    device
                        .clone()
                        .create_buffer_init(&BufferInitDescriptor {
                            label: Some("geometry_buffer"),
                            contents: encoded_uv_sphere.buffer.as_slice(),
                            usage: BufferUsages::INDEX | BufferUsages::STORAGE,
                        })
                        .into(),
                ),
            ]),
            vertices_layout: vertices_layout.into(),
            frame_data_layout: frame_data_layout.into(),
        })
    }
}

pub struct PreviewRendererResources {
    pub surface: Rc<wgpu::Surface<'static>>,
    pub render_pipeline: Rc<wgpu::RenderPipeline>,
    pub geometry_buffer: Rc<wgpu::Buffer>,
    pub geometry: Rc<EncodedSceneFile>,
    pub frame: Rc<Frame>,
    pub width: u32,
    pub height: u32,
}

impl PreviewRendererResources {
    #[cfg(target_arch = "wasm32")]
    pub fn new_from_offscreen_canvas(
        shared_resources: Rc<SharedPreviewRendererResources>,
        geometry: BuiltInGeometry,
        canvas: OffscreenCanvas,
        vertex_shader: wgpu::naga::Module,
        fragment_shader: wgpu::naga::Module,
    ) -> Result<PreviewRendererResources, Box<dyn Error>> {
        let width = canvas.width();
        let height = canvas.height();

        let surface: Rc<wgpu::Surface> = shared_resources
            .instance
            .create_surface(SurfaceTarget::OffscreenCanvas(canvas))?
            .into();
        let (geometry, geometry_buffer) = match (
            shared_resources.geometry.get(&geometry),
            shared_resources.geometry_buffers.get(&geometry),
        ) {
            (Some(geo), Some(buffer)) => Ok((geo.clone(), buffer.clone())),
            _ => Err(PreviewRendererError::GeometryTypeNotFound),
        }?;

        let vertex_shader =
            shared_resources
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Naga(Cow::Owned(vertex_shader)),
                });

        let fragment_shader =
            shared_resources
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Naga(Cow::Owned(fragment_shader)),
                });

        let pipeline_layout =
            shared_resources
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &shared_resources.vertices_layout,
                        &shared_resources.frame_data_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let swapchain_capabilities = surface.get_capabilities(&shared_resources.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = shared_resources
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: "vertexMain",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
                    entry_point: "fragmentMain",
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            })
            .into();

        Ok(PreviewRendererResources {
            surface,
            render_pipeline,
            geometry_buffer,
            geometry,
            frame: Frame::default(width, height).into(),
            width,
            height,
        })
    }

    pub fn render(&mut self, shared_resources: Rc<SharedPreviewRendererResources>) {
        // Create the logical device and command queue

        let model_matrix = Mat4::default();
        let model_view_matrix = OPENGL_TO_WGPU_MATRIX * model_matrix;
        let model_transform_data = shader_layouts::model_transform::ModelTransform {
            normal_matrix: Mat3::from_mat4(model_view_matrix),
            model_matrix,
            model_view_matrix,
        };

        let model_transform_data_buffer =
            shared_resources
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some("model_transform_data_buffer"),
                    contents: bytemuck::bytes_of(&model_transform_data),
                    usage: BufferUsages::STORAGE,
                });

        let frame_data_buffer = shared_resources
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("frame_data_buffer"),
                contents: bytemuck::bytes_of(self.frame.as_ref()),
                usage: BufferUsages::STORAGE,
            });

        let mut vertices_bind_groups: Vec<Vec<BindGroup>> = vec![];
        let mut frame_data_bind_groups: Vec<BindGroup> = vec![];

        for mesh in self.geometry.meshes.iter() {
            let mut groups: Vec<BindGroup> = vec![];
            frame_data_bind_groups.push(shared_resources.device.create_bind_group(
                &BindGroupDescriptor {
                    label: Some("frame_data_bind_group"),
                    layout: &shared_resources.frame_data_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: FRAME_BINDING,
                            resource: BindingResource::Buffer(BufferBinding {
                                buffer: &frame_data_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        BindGroupEntry {
                            binding: MODEL_TRANSFORM_BINDING,
                            resource: BindingResource::Buffer(BufferBinding {
                                buffer: &model_transform_data_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                },
            ));
            for primitive in mesh.primitives.iter() {
                let vertices_bind_group =
                    shared_resources
                        .device
                        .create_bind_group(&BindGroupDescriptor {
                            label: Some("vertices_bind_group"),
                            layout: &shared_resources.vertices_layout,
                            entries: &[BindGroupEntry {
                                binding: VERTICES_BINDING,
                                resource: BindingResource::Buffer(BufferBinding {
                                    buffer: &self.geometry_buffer,
                                    offset: primitive.vertices.offset as u64,
                                    size: Some(
                                        NonZeroU64::new(primitive.vertices.size as u64).unwrap(),
                                    ),
                                }),
                            }],
                        });
                groups.push(vertices_bind_group);
            }
            vertices_bind_groups.push(groups);
        }

        let mut config = self
            .surface
            .get_default_config(&shared_resources.adapter, self.width, self.height)
            .unwrap();

        config.alpha_mode = CompositeAlphaMode::PreMultiplied;

        self.surface.configure(&shared_resources.device, &config);

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = shared_resources
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            for (i, mesh) in self.geometry.meshes.iter().enumerate() {
                rpass.set_bind_group(FRAME_GROUP, &frame_data_bind_groups[i], &[]);
                for (j, primitive) in mesh.primitives.iter().enumerate() {
                    rpass.set_bind_group(GEOMETRY_GROUP, &vertices_bind_groups[i][j], &[]);
                    rpass.set_index_buffer(
                        self.geometry_buffer.slice(
                            primitive.indices.offset as u64
                                ..(primitive.indices.offset + primitive.indices.size) as u64,
                        ),
                        wgpu::IndexFormat::Uint32,
                    );
                    rpass.draw_indexed(
                        0..(primitive.indices.size / size_of::<u32>()) as u32,
                        0,
                        0..1,
                    );
                }
            }
        }

        shared_resources.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
