#[cfg(test)]
mod gltf_encoder_tests {
    use std::mem::size_of;

    use wbbl::{
        builtin_geometry::{get_cube, get_uv_sphere},
        gltf_encoder::{EncodedGltf, EncodingError},
        shader_layouts::vertex,
    };

    #[test]
    fn test_uv_sphere() -> Result<(), EncodingError> {
        let uv_sphere = get_uv_sphere();
        let encoded_uv_sphere = EncodedGltf::encode(&uv_sphere)?;
        assert_eq!(encoded_uv_sphere.meshes.len(), 1);
        assert_eq!(encoded_uv_sphere.meshes[0].primatives.len(), 1);
        Ok(())
    }

    #[test]
    fn test_cube() -> Result<(), EncodingError> {
        let cube = get_cube();
        let encoded_cube = EncodedGltf::encode(&cube)?;
        assert_eq!(encoded_cube.meshes.len(), 1);
        assert_eq!(encoded_cube.meshes[0].primatives.len(), 1);
        assert_eq!(
            encoded_cube.meshes[0].primatives[0].indices.size,
            4 * (12 * 3)
        );
        assert_eq!(
            encoded_cube.meshes[0].primatives[0].vertices.size,
            (4 * 6 * size_of::<vertex::Vertex>())
        );
        Ok(())
    }
}
