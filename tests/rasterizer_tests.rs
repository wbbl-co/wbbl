#[cfg(test)]
mod rasterizer_tests {

    use wbbl::compute_rasterizer::generate_compute_rasterizer;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    #[test]
    fn test_buffer_to_image_shader_codegen() {
        let result = generate_compute_rasterizer(
            wbbl::intermediate_compiler_types::BaseSizeMultiplier(1.0),
            false,
        );
        let m_valid = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            Default::default(),
        )
        .validate(&result.buffer_to_image_shader);

        wgpu::naga::back::wgsl::write_string(
            &result.buffer_to_image_shader,
            &m_valid.unwrap(),
            wgpu::naga::back::wgsl::WriterFlags::empty(),
        )
        .unwrap();
    }

    #[test]
    fn test_primary_shader_codegen() {
        let result = generate_compute_rasterizer(
            wbbl::intermediate_compiler_types::BaseSizeMultiplier(1.0),
            false,
        );
        let m_valid = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            Default::default(),
        )
        .validate(&result.primary_shader);

        wgpu::naga::back::wgsl::write_string(
            &result.primary_shader,
            &m_valid.unwrap(),
            wgpu::naga::back::wgsl::WriterFlags::empty(),
        )
        .unwrap();
    }
}
