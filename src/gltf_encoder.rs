use bytemuck::{Pod, PodCastError};
use glam::{Vec2, Vec3A};
use gltf::{
    accessor::{sparse::Sparse, DataType},
    Accessor, Gltf, Primitive, Semantic,
};

use crate::{
    model_scene_file_abstractions::{
        EncodedMesh, EncodedPrimative, EncodedSceneFile, UnboundBufferSlice,
    },
    shader_layouts::vertex::Vertex,
};

#[derive(Debug)]
pub enum EncodingError {
    MissingBlob,
    MissingIndices,
    UriReferences,
    SignedIndexType,
    PodCastError(PodCastError),
    GltfError(gltf::Error),
    ExpectedInteger,
    ExpectedFloat,
    Unknown,
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

fn reconstruct_sparse(
    buffer_blobs: &Vec<&[u8]>,
    accessor: &Accessor,
    sparse: &Sparse,
) -> Result<Vec<u8>, EncodingError> {
    let component_size = accessor.data_type().size() * accessor.dimensions().multiplicity();
    let result_length = component_size * accessor.count();
    let mut result: Vec<u8> = [0..result_length].iter().map(|_| 0 as u8).collect();

    let indices = (&sparse).indices();
    let count = (&sparse).count();
    let values = (&sparse).values();

    let indices_offset = indices.offset();
    let indices_view_buffer_index = indices.view().buffer().index();
    let indices: Vec<u32> = match indices.index_type() {
        gltf::accessor::sparse::IndexType::U8 => convert_to_u32(
            &buffer_blobs[indices_view_buffer_index][indices_offset..indices_offset + count],
            DataType::U8,
        ),
        gltf::accessor::sparse::IndexType::U16 => convert_to_u32(
            &buffer_blobs[indices_view_buffer_index][indices_offset..indices_offset + 2 * count],
            DataType::U16,
        ),
        gltf::accessor::sparse::IndexType::U32 => convert_to_u32(
            &buffer_blobs[indices_view_buffer_index][indices_offset..indices_offset + 4 * count],
            DataType::U32,
        ),
    }?;

    let value_view = values.view();
    let values_offset = values.offset();

    let value_view_blob = &buffer_blobs[value_view.buffer().index()];
    let value_view_length = value_view.length();
    let value_view_data = if let Some(stride) = value_view.stride() {
        let mut i = 0;
        let mut value_view_data: Vec<u8> = vec![];
        while i < value_view_length {
            let pos = values_offset + i;
            let mut input_slice = value_view_blob[pos..pos + component_size].to_vec();
            value_view_data.append(&mut input_slice);
            i = i + stride;
        }
        value_view_data
    } else {
        value_view_blob[values_offset..values_offset + value_view_length].to_vec()
    };

    for i in 0..count {
        let index = indices[i];
        let result_byte_offset = (index as usize) * component_size;
        for j in 0..component_size {
            let position: usize = result_byte_offset + j;
            result[position] = value_view_data[i * component_size + j] as u8;
        }
    }

    Ok(result)
}

fn gather_data<Type>(
    buffer_blobs: &Vec<&[u8]>,
    accessor: &Accessor,
    converter: fn(&[u8], DataType) -> Result<Vec<Type>, EncodingError>,
) -> Result<Vec<Type>, EncodingError> {
    if let Some(view) = accessor.view() {
        let length = view.length();
        let offset = view.offset();
        let size = accessor.data_type().size() * accessor.dimensions().multiplicity();
        if let Some(stride) = view.stride() {
            let mut result: Vec<Type> = vec![];
            let mut i = 0;
            while i < length {
                let pos = offset + i;
                let input_slice = &buffer_blobs[view.buffer().index()][pos..pos + size];
                let mut items = converter(&input_slice, accessor.data_type())?;
                result.append(&mut items);
                i = i + stride;
            }
            Ok(result)
        } else {
            let cast_slice: Vec<Type> = converter(
                &buffer_blobs[view.buffer().index()][offset..offset + length],
                accessor.data_type(),
            )?;
            Ok(cast_slice)
        }
    } else if let Some(sparse) = accessor.sparse() {
        let buffer = reconstruct_sparse(&buffer_blobs, &accessor, &sparse)?;
        let cast_slice: Vec<Type> = converter(&buffer, accessor.data_type())?;
        Ok(cast_slice)
    } else {
        Err(EncodingError::Unknown)
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

fn gather_vec3(
    buffer_blobs: &Vec<&[u8]>,
    accessor: &Accessor,
) -> Result<Vec<Vec3A>, EncodingError> {
    let intermediate: Vec<f32> = gather_data(&buffer_blobs, &accessor, convert_to_f32)?;
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

fn gather_vec2(buffer_blobs: &Vec<&[u8]>, accessor: &Accessor) -> Result<Vec<Vec2>, EncodingError> {
    let intermediate: Vec<f32> = gather_data(&buffer_blobs, &accessor, convert_to_f32)?;
    let mut results: Vec<Vec2> = vec![];
    for i in (0..intermediate.len() - 1).step_by(2) {
        results.push(Vec2 {
            x: intermediate[i],
            y: intermediate[i + 1],
        })
    }
    Ok(results)
}

fn get_vertices(
    buffer_blobs: &Vec<&[u8]>,
    primative: &Primitive,
) -> Result<Vec<Vertex>, EncodingError> {
    let mut positions: Vec<Vec3A> = vec![];
    let mut normals: Vec<Vec3A> = vec![];
    let mut tex_coords: Vec<Vec2> = vec![];
    let mut tangents: Vec<Vec3A> = vec![];

    let mut result: Vec<Vertex> = vec![];
    for (semantic, accessor) in primative.attributes() {
        match semantic {
            Semantic::Positions => {
                positions = gather_vec3(buffer_blobs, &accessor)?;
            }
            Semantic::Normals => {
                normals = gather_vec3(buffer_blobs, &accessor)?;
            }
            Semantic::Tangents => {
                tangents = gather_vec3(buffer_blobs, &accessor)?;
            }
            Semantic::TexCoords(0) => {
                tex_coords = gather_vec2(buffer_blobs, &accessor)?;
            }
            _ => {
                // DO NOTHING (for now)
            }
        }
    }
    for i in 0..positions.len() {
        let tangent = tangents
            .get(i)
            .map(|p| p.clone())
            .unwrap_or(Vec3A::default());
        let normal = normals
            .get(i)
            .map(|p| p.clone())
            .unwrap_or(Vec3A::default());
        result.push(Vertex {
            position: positions[i],
            normal,
            tangent,
            bitangent: tangent
                .normalize_or_zero()
                .cross(normal.normalize_or_zero()),
            tex_coord: tex_coords
                .get(i)
                .map(|p| p.clone())
                .unwrap_or(Vec2::default()),
        });
    }

    Ok(result)
}

pub fn encode(document: &mut Gltf) -> Result<EncodedSceneFile, EncodingError> {
    let mut buffer: Vec<u8> = vec![];
    let mut meshes: Vec<EncodedMesh> = vec![];

    let mut buffer_blobs_vec: Vec<Vec<u8>> = vec![];
    for buffer in document.buffers() {
        let buffer = gltf::buffer::Data::from_source_and_blob(
            buffer.source(),
            None,
            &mut document.blob.clone(),
        )
        .map_err(|err| EncodingError::GltfError(err))?;
        buffer_blobs_vec.push(buffer.0);
    }
    let buffer_blobs: Vec<&[u8]> = buffer_blobs_vec.iter().map(|b| b.as_slice()).collect();

    for mesh in (&document).meshes() {
        let name = mesh.name().map(|n| n.to_string());
        let mut primatives: Vec<EncodedPrimative> = vec![];
        for primative in mesh.primitives() {
            let vertices_start = buffer.len();
            let vertices = get_vertices(&buffer_blobs, &primative)?;
            for vertex in vertices.iter() {
                let mut vertex = bytemuck::bytes_of(vertex).to_vec();
                buffer.append(&mut vertex);
            }
            let vertices_end = buffer.len();

            if let Some(indices) = primative.indices() {
                let indices_start = buffer.len();
                let indices: Vec<u32> = gather_data(&buffer_blobs, &indices, convert_to_u32)?;
                for index in indices.iter() {
                    let mut index = bytemuck::bytes_of(index).to_vec();
                    buffer.append(&mut index);
                }
                let indices_end = buffer.len();
                primatives.push(EncodedPrimative {
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
        meshes.push(EncodedMesh {
            name,
            primitives: primatives,
        });
    }

    Ok(EncodedSceneFile { buffer, meshes })
}
