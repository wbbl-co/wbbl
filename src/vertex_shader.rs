use crate::{
    compiler_constants::{
        FRAME_BINDING, FRAME_GROUP, GEOMETRY_GROUP, MODEL_TRANSFORM_BINDING, VERTICES_BINDING,
    },
    shader_layouts::{frame, model_transform, vertex, vertex_out},
    utils::make_span,
};
use wgpu::naga::{
    AddressSpace, Arena, ArraySize, BinaryOperator, Binding, EntryPoint, Expression, Function,
    FunctionArgument, FunctionResult, GlobalVariable, Literal, LocalVariable, MathFunction, Module,
    Range, ResourceBinding, Scalar, ScalarKind, Statement, StorageAccess, Type, TypeInner,
    VectorSize,
};

pub fn make_vertex_shader_module() -> Module {
    let mut shader: Module = Default::default();

    let type_uint32 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Scalar(Scalar {
                kind: ScalarKind::Uint,
                width: 4,
            }),
        },
        make_span(line!()),
    );

    let type_matrix_4 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Matrix {
                columns: VectorSize::Quad,
                rows: VectorSize::Quad,
                scalar: Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                },
            },
        },
        make_span(line!()),
    );
    let type_matrix_3 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Matrix {
                columns: VectorSize::Tri,
                rows: VectorSize::Tri,
                scalar: Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                },
            },
        },
        make_span(line!()),
    );
    let type_float32_2 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Bi,
                scalar: Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                },
            },
        },
        make_span(line!()),
    );
    let type_float32_3 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Tri,
                scalar: Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                },
            },
        },
        make_span(line!()),
    );
    let type_float32_4 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Quad,
                scalar: Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                },
            },
        },
        make_span(line!()),
    );
    let type_frame_data = shader.types.insert(
        frame::make_naga_type(type_matrix_4, type_float32_2, type_float32_3),
        make_span(line!()),
    );
    let type_model_transform_data = shader.types.insert(
        model_transform::make_naga_type(type_matrix_4, type_matrix_3),
        make_span(line!()),
    );

    let type_vertex_out_data = shader.types.insert(
        vertex_out::make_naga_type(type_float32_4, type_float32_3, type_float32_2),
        make_span(line!()),
    );

    let type_vertex_data = shader.types.insert(
        vertex::make_naga_type(type_float32_3, type_float32_2),
        make_span(line!()),
    );

    let type_vertices_array = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Array {
                base: type_vertex_data,
                size: ArraySize::Dynamic,
                stride: vertex::VERTEX_STRIDE,
            },
        },
        make_span(line!()),
    );

    let global_variable_frame_data = shader.global_variables.append(
        GlobalVariable {
            name: Some("frame".to_owned()),
            space: AddressSpace::Storage {
                access: StorageAccess::LOAD,
            },
            binding: Some(ResourceBinding {
                group: FRAME_GROUP,
                binding: FRAME_BINDING,
            }),
            ty: type_frame_data,
            init: None,
        },
        make_span(line!()),
    );

    let global_variable_vertex_data_array = shader.global_variables.append(
        GlobalVariable {
            name: Some("vertices".to_owned()),
            space: AddressSpace::Storage {
                access: StorageAccess::LOAD,
            },
            binding: Some(ResourceBinding {
                group: GEOMETRY_GROUP,
                binding: VERTICES_BINDING,
            }),
            ty: type_vertices_array,
            init: None,
        },
        make_span(line!()),
    );

    let global_variable_model_transform_data = shader.global_variables.append(
        GlobalVariable {
            name: Some("model".to_owned()),
            space: AddressSpace::Storage {
                access: StorageAccess::LOAD,
            },
            binding: Some(ResourceBinding {
                group: FRAME_GROUP,
                binding: MODEL_TRANSFORM_BINDING,
            }),
            ty: type_model_transform_data,
            init: None,
        },
        make_span(line!()),
    );

    let mut main_function = Function {
        name: Some("vertexMain".to_owned()),
        arguments: vec![FunctionArgument {
            name: Some("vertex_index".to_owned()),
            ty: type_uint32,
            binding: Some(Binding::BuiltIn(wgpu::naga::BuiltIn::VertexIndex)),
        }],
        result: Some(FunctionResult {
            ty: type_vertex_out_data,
            binding: None,
        }),
        local_variables: Arena::new(),
        expressions: Arena::new(),
        named_expressions: Default::default(),
        body: Default::default(),
    };

    let constant_one_float = main_function
        .expressions
        .append(Expression::Literal(Literal::F32(1.0)), make_span(line!()));

    let local_variable_vert_out = main_function.local_variables.append(
        LocalVariable {
            name: Some("vertex_out".to_owned()),
            ty: type_vertex_out_data,
            init: None,
        },
        make_span(line!()),
    );

    let vertex_index = main_function
        .expressions
        .append(Expression::FunctionArgument(0), make_span(line!()));

    let local_variable_vert_out_ptr = main_function.expressions.append(
        Expression::LocalVariable(local_variable_vert_out),
        make_span(line!()),
    );

    let global_vertices_ptr = main_function.expressions.append(
        Expression::GlobalVariable(global_variable_vertex_data_array),
        make_span(line!()),
    );

    let vertex_data_ptr = main_function.expressions.append(
        Expression::Access {
            base: global_vertices_ptr,
            index: vertex_index,
        },
        make_span(line!()),
    );

    let vertex_data = main_function.expressions.append(
        Expression::Load {
            pointer: vertex_data_ptr,
        },
        make_span(line!()),
    );
    main_function
        .named_expressions
        .insert(vertex_data, "vertex_in".to_owned());

    // Write TexCoord through
    let tex_coord = main_function.expressions.append(
        Expression::AccessIndex {
            base: vertex_data,
            index: vertex::TEX_COORD_INDEX,
        },
        make_span(line!()),
    );

    let local_variable_vert_out_tex_coord_ptr = main_function.expressions.append(
        Expression::AccessIndex {
            base: local_variable_vert_out_ptr,
            index: vertex_out::TEX_COORD_INDEX,
        },
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(
            vertex_data_ptr,
            local_variable_vert_out_tex_coord_ptr,
        )),
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Store {
            pointer: local_variable_vert_out_tex_coord_ptr,
            value: tex_coord,
        },
        make_span(line!()),
    );

    // Start doing matrix transformations on the other values
    let global_arg_frame_data_ptr = main_function.expressions.append(
        Expression::GlobalVariable(global_variable_frame_data),
        make_span(line!()),
    );
    let global_arg_model_transform_data_ptr = main_function.expressions.append(
        Expression::GlobalVariable(global_variable_model_transform_data),
        make_span(line!()),
    );
    let loaded_frame_data = main_function.expressions.append(
        Expression::Load {
            pointer: global_arg_frame_data_ptr,
        },
        make_span(line!()),
    );
    main_function
        .named_expressions
        .insert(loaded_frame_data, "frame_data".to_owned());

    let loaded_model_transform_data = main_function.expressions.append(
        Expression::Load {
            pointer: global_arg_model_transform_data_ptr,
        },
        make_span(line!()),
    );

    main_function
        .named_expressions
        .insert(loaded_model_transform_data, "model_transform".to_owned());

    let position_in = main_function.expressions.append(
        Expression::AccessIndex {
            base: vertex_data,
            index: vertex::POSITION_INDEX,
        },
        make_span(line!()),
    );

    let position_in = main_function.expressions.append(
        Expression::Compose {
            ty: type_float32_4,
            components: vec![position_in, constant_one_float],
        },
        make_span(line!()),
    );

    main_function
        .named_expressions
        .insert(position_in, "position_in".to_owned());

    // Calculate World Position
    let model_matrix = main_function.expressions.append(
        Expression::AccessIndex {
            base: loaded_model_transform_data,
            index: model_transform::MODEL_MATRIX_INDEX,
        },
        make_span(line!()),
    );

    let world_pos = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: position_in,
            right: model_matrix,
        },
        make_span(line!()),
    );

    let local_variable_vert_out_world_position_ptr = main_function.expressions.append(
        Expression::AccessIndex {
            base: local_variable_vert_out_ptr,
            index: vertex_out::WORLD_POSITION_INDEX,
        },
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(
            loaded_frame_data,
            local_variable_vert_out_world_position_ptr,
        )),
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Store {
            pointer: local_variable_vert_out_world_position_ptr,
            value: world_pos,
        },
        make_span(line!()),
    );

    // Calculate position in clip space
    let projection_matrix = main_function.expressions.append(
        Expression::AccessIndex {
            base: loaded_frame_data,
            index: frame::PROJECTION_VIEW_MATRIX_INDEX,
        },
        make_span(line!()),
    );

    let model_view_matrix = main_function.expressions.append(
        Expression::AccessIndex {
            base: loaded_model_transform_data,
            index: model_transform::MODEL_VIEW_MATRIX_INDEX,
        },
        make_span(line!()),
    );

    let position_times_model_view_matrix_id = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: model_view_matrix,
            right: position_in,
        },
        make_span(line!()),
    );

    let clip_space_position_id = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: projection_matrix,
            right: position_times_model_view_matrix_id,
        },
        make_span(line!()),
    );

    let local_variable_vert_out_builtin_position_ptr = main_function.expressions.append(
        Expression::AccessIndex {
            base: local_variable_vert_out_ptr,
            index: vertex_out::BUILT_IN_POSITION_INDEX,
        },
        make_span(line!()),
    );

    let local_variable_vert_out_position_ptr = main_function.expressions.append(
        Expression::AccessIndex {
            base: local_variable_vert_out_ptr,
            index: vertex_out::POSITION_INDEX,
        },
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(
            projection_matrix,
            local_variable_vert_out_position_ptr,
        )),
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Store {
            pointer: local_variable_vert_out_position_ptr,
            value: clip_space_position_id,
        },
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Store {
            pointer: local_variable_vert_out_builtin_position_ptr,
            value: clip_space_position_id,
        },
        make_span(line!()),
    );

    // Rotate the tangents, bitangents and normals by the normal matrix
    let normal_matrix = main_function.expressions.append(
        Expression::AccessIndex {
            base: loaded_model_transform_data,
            index: model_transform::NORMAL_MATRIX_INDEX,
        },
        make_span(line!()),
    );

    let base_normal = main_function.expressions.append(
        Expression::AccessIndex {
            base: vertex_data,
            index: vertex::NORMAL_INDEX,
        },
        make_span(line!()),
    );

    let unormalized_normal = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: normal_matrix,
            right: base_normal,
        },
        make_span(line!()),
    );

    let normal = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Normalize,
            arg: unormalized_normal,
            arg1: None,
            arg2: None,
            arg3: None,
        },
        make_span(line!()),
    );

    let local_variable_vert_out_normal_ptr = main_function.expressions.append(
        Expression::AccessIndex {
            base: local_variable_vert_out_ptr,
            index: vertex_out::NORMAL_INDEX,
        },
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(
            normal_matrix,
            local_variable_vert_out_normal_ptr,
        )),
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Store {
            pointer: local_variable_vert_out_normal_ptr,
            value: normal,
        },
        make_span(line!()),
    );

    // Tangent
    let base_tangent = main_function.expressions.append(
        Expression::AccessIndex {
            base: vertex_data,
            index: vertex::TANGENT_INDEX,
        },
        make_span(line!()),
    );

    let unormalized_tangent = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: normal_matrix,
            right: base_tangent,
        },
        make_span(line!()),
    );

    let tangent = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Normalize,
            arg: unormalized_tangent,
            arg1: None,
            arg2: None,
            arg3: None,
        },
        make_span(line!()),
    );

    let local_variable_vert_out_tangent_ptr = main_function.expressions.append(
        Expression::AccessIndex {
            base: local_variable_vert_out_ptr,
            index: vertex_out::TANGENT_INDEX,
        },
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(
            base_tangent,
            local_variable_vert_out_tangent_ptr,
        )),
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Store {
            pointer: local_variable_vert_out_tangent_ptr,
            value: tangent,
        },
        make_span(line!()),
    );

    // Bitangent
    let base_bitangent = main_function.expressions.append(
        Expression::AccessIndex {
            base: vertex_data,
            index: vertex::BITANGENT_INDEX,
        },
        make_span(line!()),
    );

    let unormalized_bitangent = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: normal_matrix,
            right: base_bitangent,
        },
        make_span(line!()),
    );

    let bitangent = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Normalize,
            arg: unormalized_bitangent,
            arg1: None,
            arg2: None,
            arg3: None,
        },
        make_span(line!()),
    );

    let local_variable_vert_out_bitangent_ptr = main_function.expressions.append(
        Expression::AccessIndex {
            base: local_variable_vert_out_ptr,
            index: vertex_out::BITANGENT_INDEX,
        },
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(
            base_bitangent,
            local_variable_vert_out_bitangent_ptr,
        )),
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Store {
            pointer: local_variable_vert_out_bitangent_ptr,
            value: bitangent,
        },
        make_span(line!()),
    );

    // Load the vertex out data and return
    let loaded_vertex_out_data = main_function.expressions.append(
        Expression::Load {
            pointer: local_variable_vert_out_ptr,
        },
        make_span(line!()),
    );
    main_function
        .named_expressions
        .insert(loaded_vertex_out_data, "result".to_owned());

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(
            loaded_vertex_out_data,
            loaded_vertex_out_data,
        )),
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Return {
            value: Some(loaded_vertex_out_data),
        },
        make_span(line!()),
    );

    shader.entry_points.push(EntryPoint {
        name: "vertexMain".to_owned(),
        stage: wgpu::naga::ShaderStage::Vertex,
        early_depth_test: None,
        workgroup_size: [0, 0, 0],
        function: main_function,
    });
    shader
}
