use gltf::Gltf;

pub fn get_uv_sphere() -> Result<Gltf, gltf::Error> {
    let uv_sphere = include_bytes!("uv_sphere.glb").as_slice();
    Gltf::from_slice(uv_sphere)
}

pub fn get_cube() -> Result<Gltf, gltf::Error> {
    let cube = include_bytes!("cube.glb").as_slice();
    Gltf::from_slice(cube)
}
