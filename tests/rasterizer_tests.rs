#[cfg(test)]
mod rasterizer_tests {
    use std::io::Error;

    use wbbl::compute_rasterizer::generate_compute_rasterizer;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_buffer_to_image_shader_codegen() {
        let result = generate_compute_rasterizer(
            wbbl::intermediate_compiler_types::BaseSizeMultiplier(1.0),
            false,
        );
        let m_valid =
            naga::valid::Validator::new(naga::valid::ValidationFlags::empty(), Default::default())
                .validate(&result.buffer_to_image_shader);
        println!("// ModuleInfo:\n{:?}", m_valid);

        println!(
            "// WGSL:\n{}",
            naga::back::wgsl::write_string(
                &result.buffer_to_image_shader,
                &m_valid.unwrap(),
                naga::back::wgsl::WriterFlags::empty()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_primary_shader_codegen() {
        let result = generate_compute_rasterizer(
            wbbl::intermediate_compiler_types::BaseSizeMultiplier(1.0),
            false,
        );
        let m_valid =
            naga::valid::Validator::new(naga::valid::ValidationFlags::empty(), Default::default())
                .validate(&result.primary_shader);

        println!(
            "// WGSL:\n{}",
            naga::back::wgsl::write_string(
                &result.primary_shader,
                &m_valid.unwrap(),
                naga::back::wgsl::WriterFlags::empty()
            )
            .unwrap()
        );
    }
}
