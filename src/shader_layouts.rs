pub mod vertex {
    use glam::{Vec2, Vec3};
    use wgpu::naga::{Handle, StructMember, Type, TypeInner};

    #[repr(align(16))]
    pub struct Vertex {
        pub position: Vec3,
        pub normal: Vec3,
        pub tangent: Vec3,
        pub bitangent: Vec3,
        pub tex_coord: Vec2,
    }

    pub const POSITION_INDEX: u32 = 0;
    pub const NORMAL_INDEX: u32 = 1;
    pub const TANGENT_INDEX: u32 = 2;
    pub const BITANGENT_INDEX: u32 = 3;
    pub const TEX_COORD_INDEX: u32 = 4;
    pub const VERTEX_STRIDE: u32 = 80;

    pub fn make_naga_type(type_float32_3: Handle<Type>, type_float32_2: Handle<Type>) -> Type {
        Type {
            name: Some("Vertex".to_owned()),
            inner: TypeInner::Struct {
                members: vec![
                    StructMember {
                        name: Some("position".to_owned()),
                        ty: type_float32_3,
                        binding: None,
                        offset: 0,
                    },
                    StructMember {
                        name: Some("normal".to_owned()),
                        ty: type_float32_3,
                        binding: None,
                        offset: 16,
                    },
                    StructMember {
                        name: Some("tangent".to_owned()),
                        ty: type_float32_3,
                        binding: None,
                        offset: 32,
                    },
                    StructMember {
                        name: Some("bitangent".to_owned()),
                        ty: type_float32_3,
                        binding: None,
                        offset: 48,
                    },
                    StructMember {
                        name: Some("tex_coord".to_owned()),
                        ty: type_float32_2,
                        binding: None,
                        offset: 64,
                    },
                ],
                span: VERTEX_STRIDE,
            },
        }
    }
}

pub mod frame {
    use glam::{Mat4, Vec2, Vec3};
    use wgpu::naga::{Handle, StructMember, Type, TypeInner};

    pub const PROJECTION_MATRIX_INDEX: u32 = 0;
    pub const PROJECTION_MATRIX_INV_INDEX: u32 = 1;
    pub const VIEW_MATRIX_INDEX: u32 = 2;
    pub const VIEW_MATRIX_INV_INDEX: u32 = 3;
    pub const DEPTH_UNPROJECT_INDEX: u32 = 4;
    pub const SCREEN_TO_VIEW_SPACE_INDEX: u32 = 5;
    pub const FRAME_STRIDE: u32 = 288;

    #[repr(align(16))]
    pub struct Frame {
        // Per-frame constants.
        pub projection_matrix: Mat4,
        pub projection_matrix_inv: Mat4,
        pub view_matrix: Mat4,
        pub view_matrix_inv: Mat4,
        pub depth_unproject: Vec2,
        pub screen_to_view_space: Vec3,
    }

    pub fn make_naga_type(
        type_matrix_4: Handle<Type>,
        type_float32_2: Handle<Type>,
        type_float32_3: Handle<Type>,
    ) -> Type {
        Type {
            name: Some("Frame".to_owned()),
            inner: TypeInner::Struct {
                members: vec![
                    StructMember {
                        name: Some("projection_matrix".to_owned()),
                        ty: type_matrix_4,
                        binding: None,
                        offset: 0,
                    },
                    StructMember {
                        name: Some("projection_matrix_inv".to_owned()),
                        ty: type_matrix_4,
                        binding: None,
                        offset: 64,
                    },
                    StructMember {
                        name: Some("view_matrix".to_owned()),
                        ty: type_matrix_4,
                        binding: None,
                        offset: 128,
                    },
                    StructMember {
                        name: Some("view_matrix_inv".to_owned()),
                        ty: type_matrix_4,
                        binding: None,
                        offset: 192,
                    },
                    StructMember {
                        name: Some("depth_unproject".to_owned()),
                        ty: type_float32_2,
                        binding: None,
                        offset: 256,
                    },
                    StructMember {
                        name: Some("screen_to_view_space".to_owned()),
                        ty: type_float32_3,
                        binding: None,
                        offset: 272,
                    },
                ],
                span: FRAME_STRIDE,
            },
        }
    }
}

pub mod model_transform {
    use glam::{Mat3, Mat4};
    use wgpu::naga::{Handle, StructMember, Type, TypeInner};

    #[repr(align(16))]
    pub struct ModelTransform {
        // Per-frame constants.
        pub model_view_matrix: Mat4,
        pub normal_matrix: Mat3,
        pub model_matrix: Mat4,
    }

    pub const MODEL_VIEW_MATRIX_INDEX: u32 = 0;
    pub const NORMAL_MATRIX_INDEX: u32 = 1;
    pub const MODEL_MATRIX_INDEX: u32 = 2;

    pub const MODEL_TRANSFORM_STRIDE: u32 = 176;

    pub fn make_naga_type(type_matrix_4: Handle<Type>, type_matrix_3: Handle<Type>) -> Type {
        Type {
            name: Some("ModelTransform".to_owned()),
            inner: TypeInner::Struct {
                members: vec![
                    StructMember {
                        name: Some("model_view_matrix".to_owned()),
                        ty: type_matrix_4,
                        binding: None,
                        offset: 0,
                    },
                    StructMember {
                        name: Some("normal_matrix".to_owned()),
                        ty: type_matrix_3,
                        binding: None,
                        offset: 64,
                    },
                    StructMember {
                        name: Some("model_matrix".to_owned()),
                        ty: type_matrix_4,
                        binding: None,
                        offset: 112,
                    },
                ],
                span: MODEL_TRANSFORM_STRIDE,
            },
        }
    }
}

pub mod vertex_out {
    use wgpu::naga::{Binding, BuiltIn, Handle, Interpolation, StructMember, Type, TypeInner};

    pub const POSITION_INDEX: u32 = 1;
    pub const WORLD_POSITION_INDEX: u32 = 2;
    pub const NORMAL_INDEX: u32 = 3;
    pub const TANGENT_INDEX: u32 = 4;
    pub const BITANGENT_INDEX: u32 = 5;
    pub const TEX_COORD_INDEX: u32 = 6;
    pub const VERTEX_OUT_STRIDE: u32 = 112;
    pub fn make_naga_type(
        type_float32_4: Handle<Type>,
        type_float32_3: Handle<Type>,
        type_float32_2: Handle<Type>,
    ) -> Type {
        Type {
            name: Some("VertexOut".to_owned()),
            inner: TypeInner::Struct {
                members: vec![
                    StructMember {
                        name: Some("builtin_position".to_owned()),
                        ty: type_float32_4,
                        binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: true })),
                        offset: 0,
                    },
                    StructMember {
                        name: Some("position".to_owned()),
                        ty: type_float32_4,
                        binding: Some(Binding::Location {
                            location: POSITION_INDEX - 1,
                            second_blend_source: false,
                            interpolation: Some(Interpolation::Perspective),
                            sampling: None,
                        }),
                        offset: 16,
                    },
                    StructMember {
                        name: Some("world_position".to_owned()),
                        ty: type_float32_4,
                        binding: Some(Binding::Location {
                            location: WORLD_POSITION_INDEX - 1,
                            second_blend_source: false,
                            interpolation: Some(Interpolation::Perspective),
                            sampling: None,
                        }),
                        offset: 32,
                    },
                    StructMember {
                        name: Some("normal".to_owned()),
                        ty: type_float32_3,
                        binding: Some(Binding::Location {
                            location: NORMAL_INDEX - 1,
                            second_blend_source: false,
                            interpolation: Some(Interpolation::Perspective),
                            sampling: None,
                        }),
                        offset: 48,
                    },
                    StructMember {
                        name: Some("tangent".to_owned()),
                        ty: type_float32_3,
                        binding: Some(Binding::Location {
                            location: TANGENT_INDEX - 1,
                            second_blend_source: false,
                            interpolation: Some(Interpolation::Perspective),
                            sampling: None,
                        }),
                        offset: 64,
                    },
                    StructMember {
                        name: Some("bitangent".to_owned()),
                        ty: type_float32_3,
                        binding: Some(Binding::Location {
                            location: BITANGENT_INDEX - 1,
                            second_blend_source: false,
                            interpolation: Some(Interpolation::Perspective),
                            sampling: None,
                        }),
                        offset: 80,
                    },
                    StructMember {
                        name: Some("tex_coord".to_owned()),
                        ty: type_float32_2,
                        binding: Some(Binding::Location {
                            location: TEX_COORD_INDEX - 1,
                            second_blend_source: false,
                            interpolation: Some(Interpolation::Perspective),
                            sampling: None,
                        }),
                        offset: 96,
                    },
                ],
                span: VERTEX_OUT_STRIDE,
            },
        }
    }
}
