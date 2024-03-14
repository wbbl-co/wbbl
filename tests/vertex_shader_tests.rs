#[cfg(test)]
mod vertex_shader_tests {

    use wbbl::{
        test_fragment_shader::make_fragment_shader_module, vertex_shader::make_vertex_shader_module,
    };

    #[test]
    fn test_vertex_shader_codegen() {
        let result = make_vertex_shader_module();
        let m_valid = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            Default::default(),
        )
        .validate(&result);

        println!(
            "{}",
            wgpu::naga::back::wgsl::write_string(
                &result,
                &m_valid.unwrap(),
                wgpu::naga::back::wgsl::WriterFlags::empty(),
            )
            .unwrap()
        );
    }

    #[test]
    fn test_fragment_shader_codegen() {
        let result = make_fragment_shader_module();
        let m_valid = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            Default::default(),
        )
        .validate(&result);

        println!(
            "{}",
            wgpu::naga::back::wgsl::write_string(
                &result,
                &m_valid.unwrap(),
                wgpu::naga::back::wgsl::WriterFlags::empty(),
            )
            .unwrap()
        );
    }
}
