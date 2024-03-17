use glam::{Mat3, Mat4, Vec2, Vec3, Vec4};
use std::{borrow::Cow, f32::consts::PI, mem::size_of, num::NonZeroU64};
use web_sys::OffscreenCanvas;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, BufferBinding, BufferBindingType, BufferUsages, CompositeAlphaMode, Device, Queue,
    ShaderStages, Surface, SurfaceTarget,
};

pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4 {
    x_axis: Vec4 {
        x: 1.0,
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
        y: 1.0,
        z: 0.5,
        w: 0.0,
    },
    w_axis: Vec4 {
        x: 0.0,
        y: 0.0,
        z: 0.5,
        w: 1.0,
    },
};

use crate::{
    builtin_geometry::get_cube,
    compiler_constants::{
        FRAME_BINDING, FRAME_GROUP, GEOMETRY_GROUP, MODEL_TRANSFORM_BINDING, VERTICES_BINDING,
    },
    gltf_encoder, shader_layouts,
    test_fragment_shader::make_fragment_shader_module,
    vertex_shader::make_vertex_shader_module,
};

fn run(adapter: Adapter, surface: Surface<'static>, device: Device, queue: Queue) {
    // Create the logical device and command queue

    // Load the shaders from disk
    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Naga(Cow::Owned(make_vertex_shader_module())),
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Naga(Cow::Owned(make_fragment_shader_module())),
    });

    let vertices_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&"vertices_layout"),
        entries: &vec![BindGroupLayoutEntry {
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
        label: Some(&"frame_data_layout"),
        entries: &vec![
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&vertices_layout, &frame_data_layout],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut cube = get_cube();

    let encoded_cube = gltf_encoder::encode(&mut cube).unwrap();

    let geometry_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("geometry_buffer"),
        contents: encoded_cube.buffer.as_slice(),
        usage: BufferUsages::INDEX | BufferUsages::STORAGE,
    });
    let fov = 60.0 * (PI / 180.0);
    let near = 0.01;
    let far = 20.0;

    let projection_matrix =
        Mat4::perspective_lh(fov, 1.0, near, far) * Mat4::from_rotation_x(-15.0 * (PI / 180.0));

    let view_matrix = Mat4::from_translation(Vec3 {
        x: 0.0,
        y: -0.5,
        z: 3.0,
    });

    let fov_scale = f32::tan(0.5 * fov) * 2.0;

    let screen_to_view_space = Vec3 {
        x: fov_scale / 500.0,
        y: -1.0 * fov_scale * 0.5 * 1.0,
        z: -fov_scale * 0.5,
    };

    let projection_view_matrix = projection_matrix * view_matrix;

    let frame_data = shader_layouts::frame::Frame {
        projection_view_matrix,
        projection_view_matrix_inv: projection_view_matrix.inverse(),
        view_matrix,
        view_matrix_inv: view_matrix.inverse(),
        depth_unproject: Vec2 {
            x: far / (far - near),
            y: (-near * far) / (near - far),
        },
        screen_to_view_space: screen_to_view_space.into(),
    };

    let frame_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("frame_data_buffer"),
        contents: &bytemuck::bytes_of(&frame_data),
        usage: BufferUsages::STORAGE,
    });
    let model_matrix = Mat4::default();
    let model_view_matrix = OPENGL_TO_WGPU_MATRIX * model_matrix;
    let model_transform_data = shader_layouts::model_transform::ModelTransform {
        normal_matrix: Mat3::from_mat4(model_view_matrix),
        model_matrix,
        model_view_matrix,
    };

    let model_transform_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("model_transform_data_buffer"),
        contents: &bytemuck::bytes_of(&model_transform_data),
        usage: BufferUsages::STORAGE,
    });

    let mut vertices_bind_groups: Vec<Vec<BindGroup>> = vec![];
    let mut frame_data_bind_groups: Vec<BindGroup> = vec![];

    for mesh in encoded_cube.meshes.iter() {
        let mut groups: Vec<BindGroup> = vec![];
        frame_data_bind_groups.push(device.create_bind_group(&BindGroupDescriptor {
            label: Some(&"frame_data_bind_group"),
            layout: &frame_data_layout,
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
        }));
        for primitive in mesh.primitives.iter() {
            let vertices_bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some(&"vertices_bind_group"),
                layout: &vertices_layout,
                entries: &[BindGroupEntry {
                    binding: VERTICES_BINDING,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &geometry_buffer,
                        offset: primitive.vertices.offset as u64,
                        size: Some(NonZeroU64::new(primitive.vertices.size as u64).unwrap()),
                    }),
                }],
            });
            groups.push(vertices_bind_group);
        }
        vertices_bind_groups.push(groups);
    }

    let mut config = surface.get_default_config(&adapter, 300, 300).unwrap();

    config.alpha_mode = CompositeAlphaMode::PreMultiplied;
    surface.configure(&device, &config);

    let frame = surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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

        rpass.set_pipeline(&render_pipeline);
        for (i, mesh) in encoded_cube.meshes.iter().enumerate() {
            rpass.set_bind_group(FRAME_GROUP, &frame_data_bind_groups[i], &[]);
            for (j, primitive) in mesh.primitives.iter().enumerate() {
                rpass.set_bind_group(GEOMETRY_GROUP, &vertices_bind_groups[i][j], &[]);
                rpass.set_index_buffer(
                    geometry_buffer.slice(
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

    queue.submit(Some(encoder.finish()));
    frame.present();
}

pub async fn render_preview(canvas: OffscreenCanvas, instance: wgpu::Instance) {
    let surface = instance
        .create_surface(SurfaceTarget::OffscreenCanvas(canvas))
        .unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
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
        .await
        .expect("Failed to create device");

    run(adapter, surface, device, queue);
}
