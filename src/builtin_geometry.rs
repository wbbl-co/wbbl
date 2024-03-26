use gltf::Gltf;

pub fn get_uv_sphere() -> Gltf {
    let uv_sphere = include_bytes!("uv_sphere.glb").as_slice();
    Gltf::from_slice(uv_sphere).unwrap()
}

pub fn get_cube() -> Gltf {
    let cube = include_bytes!("cube.glb").as_slice();
    Gltf::from_slice(cube).unwrap()
}

#[derive(Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum BuiltInGeometry {
    Cube,
    UVSphere,
}
