#[cfg(test)]
mod builtin_geometry_tests {
    use wbbl::builtin_geometry::{get_cube, get_uv_sphere};

    #[test]
    fn test_uv_sphere() -> Result<(), gltf::Error> {
        let uv_sphere = get_uv_sphere()?;
        let mesh = uv_sphere.meshes().next().unwrap();
        let mesh = mesh.primitives().next().unwrap();
        let view = mesh.indices().unwrap().view().unwrap();
        match view.buffer().source() {
            gltf::buffer::Source::Bin => Ok(()),
            gltf::buffer::Source::Uri(_) => Err(gltf::Error::UnsupportedScheme),
        }
    }

    #[test]
    fn test_cube() -> Result<(), gltf::Error> {
        let cube = get_cube()?;
        let mesh = cube.meshes().next().unwrap();
        let mesh = mesh.primitives().next().unwrap();
        let view = mesh.indices().unwrap().view().unwrap();
        let offset = view.offset();
        let length = view.length();
        match view.buffer().source() {
            gltf::buffer::Source::Bin => {
                if let Some(blob) = cube.blob {
                    {
                        let cast_vec: &[u16] =
                            bytemuck::cast_slice(&blob[offset..(offset + length)]);
                        println!("{:?}", cast_vec);
                    }
                };
            }
            gltf::buffer::Source::Uri(_) => {
                assert!(false, "Data should be embedded for this view");
            }
        }
        Ok(())
    }
}
