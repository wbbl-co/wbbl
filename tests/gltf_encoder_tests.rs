#[cfg(test)]
mod gltf_encoder_tests {
    use std::mem::size_of;

    use gltf::Gltf;
    use wbbl::{
        builtin_geometry::{get_cube, get_uv_sphere},
        gltf_encoder::{self, EncodingError},
        shader_layouts::vertex,
    };

    #[test]
    fn test_uv_sphere() -> Result<(), EncodingError> {
        let mut uv_sphere = get_uv_sphere();
        let encoded_uv_sphere = gltf_encoder::encode(&mut uv_sphere)?;
        assert_eq!(encoded_uv_sphere.meshes.len(), 1);
        assert_eq!(encoded_uv_sphere.meshes[0].primitives.len(), 1);
        Ok(())
    }

    #[test]
    fn test_cube() -> Result<(), EncodingError> {
        let mut cube = get_cube();
        let encoded_cube = gltf_encoder::encode(&mut cube)?;
        assert_eq!(encoded_cube.meshes.len(), 1);
        assert_eq!(encoded_cube.meshes[0].primitives.len(), 1);
        assert_eq!(
            encoded_cube.meshes[0].primitives[0].indices.size,
            4 * (12 * 3)
        );
        assert_eq!(
            encoded_cube.meshes[0].primitives[0].vertices.size,
            (4 * 6 * size_of::<vertex::Vertex>())
        );
        Ok(())
    }

    #[test]
    fn test_sparse() -> Result<(), EncodingError> {
        let sparse_file = include_bytes!("SimpleSparseAccessor.gltf").as_slice();
        let mut sparse_gltf = Gltf::from_slice(sparse_file).unwrap_or_else(|err| panic!("Error"));
        let _ = gltf_encoder::encode(&mut sparse_gltf)?;
        Ok(())
    }
}
