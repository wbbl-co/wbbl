use bytemuck::{Pod, PodCastError};
use glam::{Vec2, Vec3A};
use gltf::{accessor::DataType, Accessor, Gltf, Primitive, Semantic};

use crate::shader_layouts::vertex::Vertex;

pub struct EncodedGltfPrimative {
    pub indices: UnboundBufferSlice,
    pub vertices: UnboundBufferSlice,
}

pub struct UnboundBufferSlice {
    pub offset: usize,
    pub size: usize,
}

pub struct EncodedGltfMesh {
    pub name: Option<String>,
    pub primatives: Vec<EncodedGltfPrimative>,
}

pub struct EncodedGltf {
    pub buffer: Vec<u8>,
    pub meshes: Vec<EncodedGltfMesh>,
}

#[derive(Debug)]
pub enum EncodingError {
    MissingBlob,
    MissingIndices,
    UriReferences,
    SignedIndexType,
    SparseUnsupported,
    PodCastError(PodCastError),
    ExpectedInteger,
    ExpectedFloat,
}

fn cast_slice_with_change_in_precision<Input, Output>(
    input: &[u8],
) -> Result<Vec<Output>, EncodingError>
where
    Input: Pod,
    Output: Pod + From<Input>,
{
    let items: &[Input] =
        bytemuck::try_cast_slice(&input).map_err(|err| EncodingError::PodCastError(err))?;
    Ok(items.iter().map(|item| (*item).into()).collect())
}

fn gather_data<Type>(
    document: &Gltf,
    accessor: &Accessor,
    converter: fn(&[u8], DataType) -> Result<Vec<Type>, EncodingError>,
) -> Result<Vec<Type>, EncodingError>
where
    Type: Pod,
{
    if let Some(view) = accessor.view() {
        let blob = match view.buffer().source() {
            gltf::buffer::Source::Bin => {
                if let Some(blob) = &document.blob {
                    Ok(blob)
                } else {
                    Err(EncodingError::MissingBlob)
                }
            }
            gltf::buffer::Source::Uri(_) => Err(EncodingError::UriReferences),
        }?;

        let length = view.length();
        let offset = view.offset();
        let size = accessor.data_type().size() * accessor.dimensions().multiplicity();
        if let Some(stride) = view.stride() {
            let mut result: Vec<Type> = vec![];
            let mut i = 0;
            while i < length {
                let pos = offset + i;
                let input_slice = &blob[pos..pos + size];
                let mut items = converter(&input_slice, accessor.data_type())?;
                result.append(&mut items);
                i = i + stride;
            }
            Ok(result)
        } else {
            let cast_slice: Vec<Type> =
                converter(&blob[offset..offset + length], accessor.data_type())?;
            Ok(cast_slice)
        }
    } else {
        Err(EncodingError::SparseUnsupported)
    }
}

fn convert_to_f32(slice: &[u8], data_type: DataType) -> Result<Vec<f32>, EncodingError> {
    match data_type {
        DataType::I8 => cast_slice_with_change_in_precision::<i8, f32>(&slice),
        DataType::U8 => cast_slice_with_change_in_precision::<u8, f32>(&slice),
        DataType::I16 => cast_slice_with_change_in_precision::<i8, f32>(&slice),
        DataType::U16 => cast_slice_with_change_in_precision::<u16, f32>(&slice),
        DataType::U32 => Err(EncodingError::ExpectedFloat),
        DataType::F32 => {
            let casted: &[f32] =
                bytemuck::try_cast_slice(&slice).map_err(|err| EncodingError::PodCastError(err))?;
            Ok(casted.to_vec())
        }
    }
}
fn convert_to_u32(slice: &[u8], data_type: DataType) -> Result<Vec<u32>, EncodingError> {
    match data_type {
        DataType::I8 => {
            let casted = cast_slice_with_change_in_precision::<i8, i32>(&slice)?;
            Ok(casted.iter().map(|i| *i as u32).collect())
        }
        DataType::U8 => cast_slice_with_change_in_precision::<u8, u32>(&slice),
        DataType::I16 => {
            let casted = cast_slice_with_change_in_precision::<i16, i32>(&slice)?;
            Ok(casted.iter().map(|i| *i as u32).collect())
        }
        DataType::U16 => cast_slice_with_change_in_precision::<u16, u32>(&slice),
        DataType::U32 => {
            let casted: &[u32] =
                bytemuck::try_cast_slice(&slice).map_err(|err| EncodingError::PodCastError(err))?;
            Ok(casted.to_vec())
        }
        DataType::F32 => Err(EncodingError::ExpectedInteger),
    }
}

fn gather_vec3(document: &Gltf, accessor: &Accessor) -> Result<Vec<Vec3A>, EncodingError> {
    let intermediate: Vec<f32> = gather_data(&document, &accessor, convert_to_f32)?;
    let mut results: Vec<Vec3A> = vec![];
    for i in (0..intermediate.len() - 2).step_by(3) {
        results.push(Vec3A {
            x: intermediate[i],
            y: intermediate[i + 1],
            z: intermediate[i + 2],
        })
    }
    Ok(results)
}

fn gather_vec2(document: &Gltf, accessor: &Accessor) -> Result<Vec<Vec2>, EncodingError> {
    let intermediate: Vec<f32> = gather_data(&document, &accessor, convert_to_f32)?;
    let mut results: Vec<Vec2> = vec![];
    for i in (0..intermediate.len() - 1).step_by(2) {
        results.push(Vec2 {
            x: intermediate[i],
            y: intermediate[i + 1],
        })
    }
    Ok(results)
}

fn get_vertices(document: &Gltf, primative: &Primitive) -> Result<Vec<Vertex>, EncodingError> {
    let mut positions: Vec<Vec3A> = vec![];
    let mut normals: Vec<Vec3A> = vec![];
    let mut tex_coords: Vec<Vec2> = vec![];
    let mut tangents: Vec<Vec3A> = vec![];

    let mut result: Vec<Vertex> = vec![];
    for (semantic, accessor) in primative.attributes() {
        match semantic {
            Semantic::Positions => {
                positions = gather_vec3(&document, &accessor)?;
            }
            Semantic::Normals => {
                normals = gather_vec3(&document, &accessor)?;
            }
            Semantic::Tangents => {
                tangents = gather_vec3(&document, &accessor)?;
            }
            Semantic::TexCoords(0) => {
                tex_coords = gather_vec2(&document, &accessor)?;
            }
            _ => {
                // DO NOTHING (for now)
            }
        }
    }
    for i in 0..positions.len() {
        result.push(Vertex {
            position: positions[i],
            normal: normals
                .get(i)
                .map(|p| p.clone())
                .unwrap_or(Vec3A::default()),
            tangent: tangents
                .get(i)
                .map(|p| p.clone())
                .unwrap_or(Vec3A::default()),
            bitangent: Vec3A::default(),
            tex_coord: tex_coords
                .get(i)
                .map(|p| p.clone())
                .unwrap_or(Vec2::default()),
        });
    }

    Ok(result)
}

impl EncodedGltf {
    pub fn encode(document: &Gltf) -> Result<EncodedGltf, EncodingError> {
        let mut buffer: Vec<u8> = vec![];
        let mut meshes: Vec<EncodedGltfMesh> = vec![];
        for mesh in (&document).meshes() {
            let name = mesh.name().map(|n| n.to_string());
            let mut primatives: Vec<EncodedGltfPrimative> = vec![];
            for primative in mesh.primitives() {
                let vertices_start = buffer.len();
                let vertices = get_vertices(&document, &primative)?;
                for vertex in vertices.iter() {
                    let mut vertex = bytemuck::bytes_of(vertex).to_vec();
                    buffer.append(&mut vertex);
                }
                let vertices_end = buffer.len();

                if let Some(indices) = primative.indices() {
                    let indices_start = buffer.len();
                    let indices: Vec<u32> = gather_data(&document, &indices, convert_to_u32)?;
                    for index in indices.iter() {
                        let mut index = bytemuck::bytes_of(index).to_vec();
                        buffer.append(&mut index);
                    }
                    let indices_end = buffer.len();
                    primatives.push(EncodedGltfPrimative {
                        indices: UnboundBufferSlice {
                            offset: indices_start,
                            size: indices_end - indices_start,
                        },
                        vertices: UnboundBufferSlice {
                            offset: vertices_start,
                            size: vertices_end - vertices_start,
                        },
                    });
                } else {
                    return Err(EncodingError::MissingIndices);
                }
            }
            meshes.push(EncodedGltfMesh { name, primatives });
        }

        Ok(EncodedGltf { buffer, meshes })
    }
}
