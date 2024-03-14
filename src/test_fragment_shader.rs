use wgpu::naga::{
    Arena, Binding, EntryPoint, Expression, Function, FunctionResult, Literal, Module, Range,
    Scalar, ScalarKind, Statement, Type, TypeInner, VectorSize,
};

use crate::utils::make_span;

pub fn make_fragment_shader_module() -> Module {
    let mut shader: Module = Default::default();

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

    let mut main_function = Function {
        name: Some("fragmentMain".to_owned()),
        arguments: vec![],
        result: Some(FunctionResult {
            ty: type_float32_4,
            binding: Some(Binding::Location {
                location: 0,
                second_blend_source: false,
                interpolation: None,
                sampling: None,
            }),
        }),
        local_variables: Arena::new(),
        expressions: Arena::new(),
        named_expressions: Default::default(),
        body: Default::default(),
    };

    let gray = main_function
        .expressions
        .append(Expression::Literal(Literal::F32(0.4)), make_span(line!()));

    let one = main_function
        .expressions
        .append(Expression::Literal(Literal::F32(1.0)), make_span(line!()));

    let gray_vec4 = main_function.expressions.append(
        Expression::Compose {
            ty: type_float32_4,
            components: vec![gray, gray, gray, one],
        },
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Emit(Range::new_from_bounds(gray_vec4, gray_vec4)),
        make_span(line!()),
    );

    main_function.body.push(
        Statement::Return {
            value: Some(gray_vec4),
        },
        make_span(line!()),
    );

    shader.entry_points.push(EntryPoint {
        name: "fragmentMain".to_owned(),
        stage: wgpu::naga::ShaderStage::Fragment,
        early_depth_test: None,
        workgroup_size: [0, 0, 0],
        function: main_function,
    });
    shader
}
