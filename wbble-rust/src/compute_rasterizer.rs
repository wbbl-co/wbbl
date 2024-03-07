use naga::{
    AddressSpace, AtomicFunction, BinaryOperator, Binding::*, Block, Constant, GlobalVariable,
    ImageClass, ImageDimension, Literal, LocalVariable, MathFunction, Override, Range,
    ResourceBinding, Statement, StorageAccess, SwizzleComponent,
};
use naga::{
    Arena, ArraySize::Dynamic, BuiltIn, EntryPoint, Expression, Function, FunctionArgument, Module,
    Scalar, ScalarKind, Span, StructMember, Type, TypeInner, VectorSize,
};

use crate::compiler_constants::{
    ARGUMENTS_GROUP, COMPUTE_TEXTURE_OUTPUT_BINDING, GEOMETRY_GROUP, INDICES_BINDING,
    PER_SHADER_INPUT_OUTPUT_GROUP, TRIANGLE_INDEX_BUFFER_ARGUMENT_BINDING, VERTICES_BINDING,
};
use crate::intermediate_compiler_types::{BaseSizeMultiplier, ComputeRasterizerShader};

fn make_primary_rasterizer_module() -> Module {
    let mut shader: Module = Default::default();
    let empty_span = Span::default();

    let type_uint32 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Scalar(Scalar {
                kind: ScalarKind::Uint,
                width: 4,
            }),
        },
        empty_span.clone(),
    );

    let type_float32 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Scalar(Scalar {
                kind: ScalarKind::Float,
                width: 4,
            }),
        },
        empty_span.clone(),
    );

    let type_uint32_3 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Tri,
                scalar: Scalar {
                    kind: ScalarKind::Uint,
                    width: 4,
                },
            },
        },
        empty_span.clone(),
    );

    let type_uint32_2 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Tri,
                scalar: Scalar {
                    kind: ScalarKind::Uint,
                    width: 4,
                },
            },
        },
        empty_span.clone(),
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
        empty_span.clone(),
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
        empty_span.clone(),
    );

    let type_vertex_data = shader.types.insert(
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
                        name: Some("tex_coord".to_owned()),
                        ty: type_float32_2,
                        binding: None,
                        offset: 16,
                    },
                    StructMember {
                        name: Some("normal".to_owned()),
                        ty: type_float32_3,
                        binding: None,
                        offset: 32,
                    },
                    StructMember {
                        name: Some("tangent".to_owned()),
                        ty: type_float32_3,
                        binding: None,
                        offset: 48,
                    },
                    StructMember {
                        name: Some("bitangent".to_owned()),
                        ty: type_float32_3,
                        binding: None,
                        offset: 64,
                    },
                ],
                span: line!(),
            },
        },
        empty_span.clone(),
    );

    let type_vertices_array = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Array {
                base: type_vertex_data,
                size: Dynamic,
                stride: 80,
            },
        },
        empty_span.clone(),
    );

    let type_indices_array = shader.types.insert(
        Type {
            name: Some("Index".to_owned()),
            inner: TypeInner::Array {
                base: type_uint32,
                size: Dynamic,
                stride: 4,
            },
        },
        empty_span.clone(),
    );

    let vertex_data_array = shader.global_variables.append(
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
        empty_span.clone(),
    );

    let index_data_array = shader.global_variables.append(
        GlobalVariable {
            name: Some("indices".to_owned()),
            space: AddressSpace::Storage {
                access: StorageAccess::LOAD,
            },
            binding: Some(ResourceBinding {
                group: GEOMETRY_GROUP,
                binding: INDICES_BINDING,
            }),
            ty: type_indices_array,
            init: None,
        },
        empty_span.clone(),
    );

    let global_invocation_id = FunctionArgument {
        name: Some("invocation_id".to_owned()),
        ty: type_uint32_3,
        binding: Some(BuiltIn(BuiltIn::GlobalInvocationId)),
    };

    let mut main_function = Function {
        name: Some("computeMain".to_owned()),
        arguments: vec![global_invocation_id],
        result: None,
        local_variables: Arena::new(),
        expressions: Arena::new(),
        named_expressions: Default::default(),
        body: Default::default(),
    };

    let func_arg_global_invocation_id = main_function
        .expressions
        .append(Expression::FunctionArgument(0), empty_span.clone());

    let prelude_start = func_arg_global_invocation_id.clone();

    let global_arg_vertices_ptr = main_function.expressions.append(
        Expression::GlobalVariable(vertex_data_array),
        empty_span.clone(),
    );

    let global_arg_indices_ptr = main_function.expressions.append(
        Expression::GlobalVariable(index_data_array),
        empty_span.clone(),
    );

    let loaded_global_invocation_id = main_function.expressions.append(
        Expression::Load {
            pointer: func_arg_global_invocation_id,
        },
        empty_span.clone(),
    );

    let triangle_index = main_function.expressions.append(
        Expression::AccessIndex {
            base: loaded_global_invocation_id,
            index: 0,
        },
        empty_span.clone(),
    );

    main_function
        .named_expressions
        .insert(triangle_index, "triangle_index".to_owned());

    let constant_three_uint = shader
        .const_expressions
        .append(Expression::Literal(Literal::U32(3)), empty_span.clone());

    let constant_two_uint = shader
        .const_expressions
        .append(Expression::Literal(Literal::U32(2)), empty_span.clone());

    let constant_one_uint = shader
        .const_expressions
        .append(Expression::Literal(Literal::U32(1)), empty_span.clone());

    let output_index = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: triangle_index,
            right: constant_one_uint,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(triangle_index, "output_index".to_owned());

    let indices_index_1 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: triangle_index,
            right: constant_three_uint,
        },
        empty_span.clone(),
    );

    main_function
        .named_expressions
        .insert(indices_index_1, "indices_start".to_owned());

    let indices_index_2 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: indices_index_1,
            right: constant_one_uint,
        },
        empty_span.clone(),
    );

    let indices_index_3 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: indices_index_1,
            right: constant_two_uint,
        },
        empty_span.clone(),
    );

    let index_1_access_chain = main_function.expressions.append(
        Expression::Access {
            base: global_arg_indices_ptr,
            index: indices_index_1,
        },
        empty_span.clone(),
    );

    let index_2_access_chain = main_function.expressions.append(
        Expression::Access {
            base: global_arg_indices_ptr,
            index: indices_index_2,
        },
        empty_span.clone(),
    );

    let index_3_access_chain = main_function.expressions.append(
        Expression::Access {
            base: global_arg_indices_ptr,
            index: indices_index_3,
        },
        empty_span.clone(),
    );

    let index_1 = main_function.expressions.append(
        Expression::Load {
            pointer: index_1_access_chain,
        },
        empty_span.clone(),
    );

    let index_2 = main_function.expressions.append(
        Expression::Load {
            pointer: index_2_access_chain,
        },
        empty_span.clone(),
    );

    let index_3 = main_function.expressions.append(
        Expression::Load {
            pointer: index_3_access_chain,
        },
        empty_span.clone(),
    );

    main_function
        .named_expressions
        .insert(index_1, "index_1".to_owned());
    main_function
        .named_expressions
        .insert(index_1, "index_2".to_owned());
    main_function
        .named_expressions
        .insert(index_1, "index_3".to_owned());

    let vertex_1_access_chain = main_function.expressions.append(
        Expression::Access {
            base: global_arg_vertices_ptr,
            index: index_1,
        },
        empty_span.clone(),
    );

    let vertex_2_access_chain = main_function.expressions.append(
        Expression::Access {
            base: global_arg_vertices_ptr,
            index: index_2,
        },
        empty_span.clone(),
    );

    let vertex_3_access_chain = main_function.expressions.append(
        Expression::Access {
            base: global_arg_vertices_ptr,
            index: index_3,
        },
        empty_span.clone(),
    );

    let vertex_1 = main_function.expressions.append(
        Expression::Load {
            pointer: vertex_1_access_chain,
        },
        empty_span.clone(),
    );

    let vertex_2 = main_function.expressions.append(
        Expression::Load {
            pointer: vertex_2_access_chain,
        },
        empty_span.clone(),
    );

    let vertex_3 = main_function.expressions.append(
        Expression::Load {
            pointer: vertex_3_access_chain,
        },
        empty_span.clone(),
    );

    main_function
        .named_expressions
        .insert(vertex_1, "vertex_1".to_owned());
    main_function
        .named_expressions
        .insert(vertex_2, "vertex_2".to_owned());
    main_function
        .named_expressions
        .insert(vertex_3, "vertex_3".to_owned());

    let type_output_buffer = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Array {
                base: type_uint32,
                size: Dynamic,
                stride: 4,
            },
        },
        empty_span.clone(),
    );

    let output_buffer = shader.global_variables.append(
        GlobalVariable {
            name: Some("output_buffer".to_owned()),
            space: AddressSpace::Storage {
                access: StorageAccess::LOAD | StorageAccess::STORE,
            },
            binding: Some(ResourceBinding {
                group: PER_SHADER_INPUT_OUTPUT_GROUP,
                binding: TRIANGLE_INDEX_BUFFER_ARGUMENT_BINDING,
            }),
            ty: type_output_buffer,
            init: None,
        },
        empty_span.clone(),
    );

    let global_arg_output_buffer_ptr = main_function.expressions.append(
        Expression::GlobalVariable(output_buffer),
        empty_span.clone(),
    );

    let output_buffer_length = main_function.expressions.append(
        Expression::ArrayLength(global_arg_output_buffer_ptr),
        empty_span.clone(),
    );

    let output_width = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::ShiftRight,
            left: output_buffer_length,
            right: constant_one_uint,
        },
        empty_span.clone(),
    );

    let output_dimensions_uint = main_function.expressions.append(
        Expression::Compose {
            ty: type_uint32_2,
            components: vec![output_width, output_width],
        },
        empty_span.clone(),
    );

    let uv_1 = main_function.expressions.append(
        Expression::AccessIndex {
            base: vertex_1,
            index: 1,
        },
        empty_span.clone(),
    );
    let uv_2 = main_function.expressions.append(
        Expression::AccessIndex {
            base: vertex_2,
            index: 1,
        },
        empty_span.clone(),
    );
    let uv_3 = main_function.expressions.append(
        Expression::AccessIndex {
            base: vertex_3,
            index: 1,
        },
        empty_span.clone(),
    );

    main_function
        .named_expressions
        .insert(uv_1, "uv_1".to_owned());
    main_function
        .named_expressions
        .insert(uv_2, "uv_2".to_owned());
    main_function
        .named_expressions
        .insert(uv_3, "uv_3".to_owned());

    let min_uv1_uv2 = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Min,
            arg: uv_1,
            arg1: Some(uv_2),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    let max_uv1_uv2 = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Max,
            arg: uv_1,
            arg1: Some(uv_2),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    let min_uv = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Min,
            arg: min_uv1_uv2,
            arg1: Some(uv_3),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    let max_uv = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Max,
            arg: max_uv1_uv2,
            arg1: Some(uv_3),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(min_uv, "min_uv".to_owned());
    main_function
        .named_expressions
        .insert(min_uv, "max_uv".to_owned());

    let dimensions_float = main_function.expressions.append(
        Expression::As {
            expr: output_dimensions_uint,
            kind: ScalarKind::Float,
            convert: Some(4),
        },
        empty_span.clone(),
    );
    let max_pixel_float = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: max_uv,
            right: dimensions_float,
        },
        empty_span.clone(),
    );

    let min_pixel_float = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: min_uv,
            right: dimensions_float,
        },
        empty_span.clone(),
    );

    let min_pixel_uint = main_function.expressions.append(
        Expression::As {
            expr: min_pixel_float,
            kind: ScalarKind::Uint,
            convert: Some(4),
        },
        empty_span.clone(),
    );

    let max_pixel_ceil_float = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Ceil,
            arg: max_pixel_float,
            arg1: None,
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );

    let max_pixel_uint = main_function.expressions.append(
        Expression::As {
            expr: max_pixel_ceil_float,
            kind: ScalarKind::Uint,
            convert: Some(4),
        },
        empty_span.clone(),
    );

    let pixel_min_x = main_function.expressions.append(
        Expression::AccessIndex {
            base: min_pixel_uint,
            index: 0,
        },
        empty_span.clone(),
    );
    let pixel_min_y = main_function.expressions.append(
        Expression::AccessIndex {
            base: min_pixel_uint,
            index: 0,
        },
        empty_span.clone(),
    );

    let pixel_max_x = main_function.expressions.append(
        Expression::AccessIndex {
            base: max_pixel_uint,
            index: 0,
        },
        empty_span.clone(),
    );
    let pixel_max_y = main_function.expressions.append(
        Expression::AccessIndex {
            base: max_pixel_uint,
            index: 0,
        },
        empty_span.clone(),
    );

    let constant_one_float = shader
        .const_expressions
        .append(Expression::Literal(Literal::F32(1.0)), empty_span.clone());
    let constant_one_float_2 = shader.const_expressions.append(
        Expression::Compose {
            ty: type_float32_2,
            components: vec![constant_one_float, constant_one_float],
        },
        empty_span.clone(),
    );

    let uv_delta = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Divide,
            left: constant_one_float_2,
            right: dimensions_float,
        },
        empty_span.clone(),
    );

    let delta_x = main_function.expressions.append(
        Expression::AccessIndex {
            base: uv_delta,
            index: 0,
        },
        empty_span.clone(),
    );

    let delta_y = main_function.expressions.append(
        Expression::AccessIndex {
            base: uv_delta,
            index: 1,
        },
        empty_span.clone(),
    );

    let start_uv_x = main_function.expressions.append(
        Expression::AccessIndex {
            base: min_uv,
            index: 0,
        },
        empty_span.clone(),
    );
    let start_uv_y = main_function.expressions.append(
        Expression::AccessIndex {
            base: min_uv,
            index: 1,
        },
        empty_span.clone(),
    );

    // BEGIN SHARED VALUES FOR CALCULATING BARYCENTRIC COORDS
    let v0 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Subtract,
            left: uv_2,
            right: uv_1,
        },
        empty_span.clone(),
    );
    main_function.named_expressions.insert(v0, "v0".to_owned());

    let v1 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Subtract,
            left: uv_3,
            right: uv_1,
        },
        empty_span.clone(),
    );
    main_function.named_expressions.insert(v1, "v1".to_owned());

    let d00 = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Dot,
            arg: v0,
            arg1: Some(v0),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(d00, "d00".to_owned());

    let d01 = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Dot,
            arg: v0,
            arg1: Some(v1),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(d01, "d01".to_owned());

    let d11 = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Dot,
            arg: v1,
            arg1: Some(v1),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(d11, "d11".to_owned());

    let d00_m_d11 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: d00,
            right: d11,
        },
        empty_span.clone(),
    );

    let d01_squared = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: d01,
            right: d01,
        },
        empty_span.clone(),
    );

    let denominator = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Subtract,
            left: d00_m_d11,
            right: d01_squared,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(denominator, "denominator".to_owned());

    // END SHARED VALUES FOR CALCULATING BARYCENTRIC COORDS

    let prelude_end = denominator.clone();

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(prelude_start, prelude_end)),
        empty_span.clone(),
    );

    let constant_zero_float = shader
        .const_expressions
        .append(Expression::Literal(Literal::F32(0.0)), empty_span.clone());

    let is_denominator_zero = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Equal,
            left: denominator,
            right: constant_zero_float,
        },
        empty_span.clone(),
    );

    let mut denominator_zero_block = Block::new();
    denominator_zero_block.push(Statement::Return { value: None }, empty_span.clone());
    let mut denominator_non_zero_block = Block::new();

    let one_over_denom = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Divide,
            left: constant_one_float,
            right: denominator,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(one_over_denom, "one_over_denom".to_owned());

    let pixel_x = main_function.local_variables.append(
        LocalVariable {
            name: Some("pixel_x".to_owned()),
            ty: type_uint32,
            init: None,
        },
        empty_span.clone(),
    );

    let pixel_x_ptr = main_function
        .expressions
        .append(Expression::LocalVariable(pixel_x), empty_span.clone());

    let uv_x = main_function.local_variables.append(
        LocalVariable {
            name: Some("uv_x".to_owned()),
            ty: type_float32,
            init: None,
        },
        empty_span.clone(),
    );

    let uv_x_ptr = main_function
        .expressions
        .append(Expression::LocalVariable(uv_x), empty_span.clone());

    denominator_non_zero_block.push(
        Statement::Emit(Range::new_from_bounds(one_over_denom, uv_x_ptr)),
        empty_span.clone(),
    );

    denominator_non_zero_block.push(
        Statement::Store {
            pointer: pixel_x_ptr,
            value: pixel_min_x,
        },
        empty_span.clone(),
    );

    denominator_non_zero_block.push(
        Statement::Store {
            pointer: uv_x_ptr,
            value: start_uv_x,
        },
        empty_span.clone(),
    );

    let mut x_body = Block::new();
    let loaded_pixel_x = main_function.expressions.append(
        Expression::Load {
            pointer: pixel_x_ptr,
        },
        empty_span.clone(),
    );

    let loaded_uv_x = main_function
        .expressions
        .append(Expression::Load { pointer: uv_x_ptr }, empty_span.clone());

    let pixel_y = main_function.local_variables.append(
        LocalVariable {
            name: Some("pixel_y".to_owned()),
            ty: type_uint32,
            init: None,
        },
        empty_span.clone(),
    );

    let pixel_y_ptr = main_function
        .expressions
        .append(Expression::LocalVariable(pixel_y), empty_span.clone());

    let uv_y = main_function.local_variables.append(
        LocalVariable {
            name: Some("uv_y".to_owned()),
            ty: type_float32,
            init: None,
        },
        empty_span.clone(),
    );

    let uv_y_ptr = main_function
        .expressions
        .append(Expression::LocalVariable(uv_y), empty_span.clone());

    x_body.push(
        Statement::Emit(Range::new_from_bounds(loaded_pixel_x, uv_y_ptr)),
        empty_span.clone(),
    );

    x_body.push(
        Statement::Store {
            pointer: pixel_y_ptr,
            value: pixel_min_y,
        },
        empty_span.clone(),
    );

    x_body.push(
        Statement::Store {
            pointer: uv_y_ptr,
            value: start_uv_y,
        },
        empty_span.clone(),
    );

    let mut y_body = Block::new();
    let loaded_pixel_y = main_function.expressions.append(
        Expression::Load {
            pointer: pixel_y_ptr,
        },
        empty_span.clone(),
    );

    let loaded_uv_y = main_function
        .expressions
        .append(Expression::Load { pointer: uv_y_ptr }, empty_span.clone());

    let uv = main_function.expressions.append(
        Expression::Compose {
            ty: type_float32_2,
            components: vec![loaded_uv_x, loaded_uv_y],
        },
        empty_span.clone(),
    );

    let v2 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Subtract,
            left: uv,
            right: uv_1,
        },
        empty_span.clone(),
    );
    main_function.named_expressions.insert(v2, "v2".to_owned());

    let d20 = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Dot,
            arg: v2,
            arg1: Some(v0),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(d20, "d20".to_owned());
    let d21 = main_function.expressions.append(
        Expression::Math {
            fun: MathFunction::Dot,
            arg: v2,
            arg1: Some(v1),
            arg2: None,
            arg3: None,
        },
        empty_span.clone(),
    );
    main_function
        .named_expressions
        .insert(d21, "d21".to_owned());

    let d11_m_d20 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: d11,
            right: d20,
        },
        empty_span.clone(),
    );
    let d01_m_d21 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: d01,
            right: d21,
        },
        empty_span.clone(),
    );
    let d11_m_d20_s_d01_m_d21 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Subtract,
            left: d11_m_d20,
            right: d01_m_d21,
        },
        empty_span.clone(),
    );
    let v = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: d11_m_d20_s_d01_m_d21,
            right: one_over_denom,
        },
        empty_span.clone(),
    );
    main_function.named_expressions.insert(v, "v".to_owned());
    let d00_m_d21 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: d00,
            right: d21,
        },
        empty_span.clone(),
    );
    let d01_m_d20 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: d01,
            right: d20,
        },
        empty_span.clone(),
    );
    let d00_m_d21_s_d01_m_d20 = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Subtract,
            left: d00_m_d21,
            right: d01_m_d20,
        },
        empty_span.clone(),
    );
    let w = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: d00_m_d21_s_d01_m_d20,
            right: one_over_denom,
        },
        empty_span.clone(),
    );
    main_function.named_expressions.insert(v, "w".to_owned());

    let one_minus_v = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Subtract,
            left: constant_one_float,
            right: v,
        },
        empty_span.clone(),
    );
    let u = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Subtract,
            left: one_minus_v,
            right: w,
        },
        empty_span.clone(),
    );
    y_body.push(
        Statement::Emit(Range::new_from_bounds(loaded_pixel_y, u)),
        empty_span.clone(),
    );

    let u_is_negative = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Less,
            left: constant_zero_float,
            right: u,
        },
        empty_span.clone(),
    );

    let v_is_negative = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Less,
            left: constant_zero_float,
            right: v,
        },
        empty_span.clone(),
    );

    let w_is_negative = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Less,
            left: constant_zero_float,
            right: w,
        },
        empty_span.clone(),
    );

    let u_or_v_is_negative = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::LogicalOr,
            left: u_is_negative,
            right: v_is_negative,
        },
        empty_span.clone(),
    );

    let u_or_v_or_w_is_negative = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::LogicalOr,
            left: u_or_v_is_negative,
            right: w_is_negative,
        },
        empty_span.clone(),
    );
    let mut no_write_triangle_block = Block::new();
    no_write_triangle_block.push(Statement::Continue, empty_span.clone());
    let mut write_triangle_block = Block::new();
    let rows = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: loaded_pixel_y,
            right: output_width,
        },
        empty_span.clone(),
    );
    let pixel = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: rows,
            right: loaded_pixel_x,
        },
        empty_span.clone(),
    );
    let output_buffer_pixel_ptr = main_function.expressions.append(
        Expression::Access {
            base: global_arg_output_buffer_ptr,
            index: pixel,
        },
        empty_span.clone(),
    );
    let ignored_atomic_result = main_function.expressions.append(
        Expression::AtomicResult {
            ty: type_uint32,
            comparison: true,
        },
        empty_span.clone(),
    );
    write_triangle_block.push(
        Statement::Emit(Range::new_from_bounds(rows, ignored_atomic_result)),
        empty_span.clone(),
    );

    write_triangle_block.push(
        Statement::Atomic {
            pointer: output_buffer_pixel_ptr,
            fun: AtomicFunction::Max,
            value: output_index,
            result: ignored_atomic_result,
        },
        empty_span.clone(),
    );

    y_body.push(
        Statement::If {
            condition: u_or_v_or_w_is_negative,
            accept: no_write_triangle_block,
            reject: write_triangle_block,
        },
        empty_span.clone(),
    );
    let mut y_continuing = Block::new();
    let pixel_y_next = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: loaded_pixel_y,
            right: constant_one_float,
        },
        empty_span.clone(),
    );
    let uv_y_next = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: loaded_uv_y,
            right: delta_y,
        },
        empty_span.clone(),
    );

    y_continuing.push(
        Statement::Emit(Range::new_from_bounds(pixel_y_next, uv_y_next)),
        empty_span.clone(),
    );

    y_continuing.push(
        Statement::Store {
            pointer: pixel_y_ptr,
            value: pixel_y_next,
        },
        empty_span.clone(),
    );

    y_continuing.push(
        Statement::Store {
            pointer: uv_y_ptr,
            value: uv_y_next,
        },
        empty_span.clone(),
    );

    let y_break_if = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::GreaterEqual,
            left: pixel_y_next,
            right: pixel_max_x,
        },
        empty_span.clone(),
    );

    let y_loop = Statement::Loop {
        body: y_body,
        continuing: y_continuing,
        break_if: Some(y_break_if),
    };
    x_body.push(y_loop, empty_span.clone());

    let mut x_continuing = Block::new();
    let pixel_x_next = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: loaded_pixel_x,
            right: constant_one_float,
        },
        empty_span.clone(),
    );
    let uv_x_next = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: loaded_uv_x,
            right: delta_x,
        },
        empty_span.clone(),
    );

    x_continuing.push(
        Statement::Emit(Range::new_from_bounds(pixel_x_next, uv_x_next)),
        empty_span.clone(),
    );
    x_continuing.push(
        Statement::Store {
            pointer: pixel_x_ptr,
            value: pixel_x_next,
        },
        empty_span.clone(),
    );
    x_continuing.push(
        Statement::Store {
            pointer: uv_x_ptr,
            value: uv_x_next,
        },
        empty_span.clone(),
    );

    let x_break_if = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::GreaterEqual,
            left: pixel_x_next,
            right: pixel_max_x,
        },
        empty_span.clone(),
    );

    let x_loop = Statement::Loop {
        body: x_body,
        continuing: x_continuing,
        break_if: Some(x_break_if),
    };
    denominator_non_zero_block.push(x_loop, empty_span.clone());

    let if_statement = Statement::If {
        condition: is_denominator_zero,
        accept: denominator_zero_block,
        reject: denominator_non_zero_block,
    };
    main_function.body.push(if_statement, empty_span.clone());

    shader.entry_points.push(EntryPoint {
        name: "computeRasterizerMain".to_owned(),
        stage: naga::ShaderStage::Compute,
        early_depth_test: None,
        workgroup_size: [128, 1, 1],
        function: main_function,
    });

    shader
}

fn make_buffer_to_image_module() -> Module {
    let mut shader: Module = Default::default();
    let empty_span = Span::default();

    let type_uint32 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Scalar(Scalar {
                kind: ScalarKind::Uint,
                width: 4,
            }),
        },
        empty_span.clone(),
    );

    let type_float32 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Scalar(Scalar {
                kind: ScalarKind::Float,
                width: 4,
            }),
        },
        empty_span.clone(),
    );

    let type_uint32_3 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Tri,
                scalar: Scalar {
                    kind: ScalarKind::Uint,
                    width: 4,
                },
            },
        },
        empty_span.clone(),
    );

    let type_uint32_2 = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Tri,
                scalar: Scalar {
                    kind: ScalarKind::Uint,
                    width: 4,
                },
            },
        },
        empty_span.clone(),
    );

    let global_invocation_id = FunctionArgument {
        name: Some("invocation_id".to_owned()),
        ty: type_uint32_3,
        binding: Some(BuiltIn(BuiltIn::GlobalInvocationId)),
    };

    let mut main_function = Function {
        name: Some("computeMain".to_owned()),
        arguments: vec![global_invocation_id],
        result: None,
        local_variables: Arena::new(),
        expressions: Arena::new(),
        named_expressions: Default::default(),
        body: Default::default(),
    };

    let func_arg_global_invocation_id = main_function
        .expressions
        .append(Expression::FunctionArgument(0), empty_span.clone());

    let prelude_start = func_arg_global_invocation_id.clone();

    let loaded_global_invocation_id = main_function.expressions.append(
        Expression::Load {
            pointer: func_arg_global_invocation_id,
        },
        empty_span.clone(),
    );

    let type_output_image = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Image {
                dim: ImageDimension::D2,
                arrayed: false,
                class: ImageClass::Storage {
                    format: naga::StorageFormat::R32Sint,
                    access: StorageAccess::LOAD | StorageAccess::STORE,
                },
            },
        },
        empty_span.clone(),
    );

    let output_image = shader.global_variables.append(
        GlobalVariable {
            name: Some("output_image".to_owned()),
            space: AddressSpace::Storage {
                access: StorageAccess::LOAD | StorageAccess::STORE,
            },
            binding: Some(ResourceBinding {
                group: PER_SHADER_INPUT_OUTPUT_GROUP,
                binding: COMPUTE_TEXTURE_OUTPUT_BINDING,
            }),
            ty: type_output_image,
            init: None,
        },
        empty_span.clone(),
    );

    let type_input_buffer = shader.types.insert(
        Type {
            name: None,
            inner: TypeInner::Array {
                base: type_uint32,
                size: Dynamic,
                stride: 4,
            },
        },
        empty_span.clone(),
    );

    let input_buffer = shader.global_variables.append(
        GlobalVariable {
            name: Some("output_buffer".to_owned()),
            space: AddressSpace::Storage {
                access: StorageAccess::LOAD,
            },
            binding: Some(ResourceBinding {
                group: PER_SHADER_INPUT_OUTPUT_GROUP,
                binding: TRIANGLE_INDEX_BUFFER_ARGUMENT_BINDING,
            }),
            ty: type_input_buffer,
            init: None,
        },
        empty_span.clone(),
    );

    let global_arg_input_buffer_ptr = main_function
        .expressions
        .append(Expression::GlobalVariable(input_buffer), empty_span.clone());

    let loaded_input_buffer = main_function.expressions.append(
        Expression::Load {
            pointer: global_arg_input_buffer_ptr,
        },
        empty_span.clone(),
    );

    let global_arg_output_image_ptr = main_function
        .expressions
        .append(Expression::GlobalVariable(output_image), empty_span.clone());

    let loaded_output_image = main_function.expressions.append(
        Expression::Load {
            pointer: global_arg_output_image_ptr,
        },
        empty_span.clone(),
    );

    let output_image_dimensions = main_function.expressions.append(
        Expression::ImageQuery {
            image: loaded_output_image,
            query: naga::ImageQuery::Size { level: None },
        },
        empty_span.clone(),
    );

    let pixel = main_function.expressions.append(
        Expression::Swizzle {
            size: VectorSize::Bi,
            vector: loaded_global_invocation_id,
            pattern: [
                SwizzleComponent::X,
                SwizzleComponent::Y,
                SwizzleComponent::X,
                SwizzleComponent::X,
            ],
        },
        empty_span.clone(),
    );

    let pixel_y = main_function.expressions.append(
        Expression::AccessIndex {
            base: pixel,
            index: 1,
        },
        empty_span.clone(),
    );
    let pixel_x = main_function.expressions.append(
        Expression::AccessIndex {
            base: pixel,
            index: 0,
        },
        empty_span.clone(),
    );

    let image_width = main_function.expressions.append(
        Expression::AccessIndex {
            base: output_image_dimensions,
            index: 0,
        },
        empty_span.clone(),
    );
    let row_offset = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Multiply,
            left: image_width,
            right: pixel_y,
        },
        empty_span.clone(),
    );
    let buffer_index = main_function.expressions.append(
        Expression::Binary {
            op: BinaryOperator::Add,
            left: row_offset,
            right: pixel_x,
        },
        empty_span.clone(),
    );

    let triangle_index = main_function.expressions.append(
        Expression::Access {
            base: loaded_input_buffer,
            index: buffer_index,
        },
        empty_span.clone(),
    );

    let prelude_end = triangle_index;

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(prelude_start, prelude_end)),
        empty_span.clone(),
    );

    main_function.body.push(
        Statement::ImageStore {
            image: loaded_output_image,
            coordinate: pixel,
            array_index: None,
            value: triangle_index,
        },
        empty_span.clone(),
    );
    shader
}

pub fn generate_compute_rasterizer(
    output_size_multiplier: BaseSizeMultiplier,
    generate_mip_maps: bool,
) -> ComputeRasterizerShader {
    ComputeRasterizerShader {
        primary_shader: make_primary_rasterizer_module(),
        buffer_to_image_shader: make_buffer_to_image_module(),
        output_size_multiplier,
        generate_mip_maps,
    }
}
