//! Bine engine
//!
//! Author: BEKs => 14.06.2026
//!
//! This module manages all resources

use wgpu::util::DeviceExt;

use crate::renderer::{Texture, model};

use std::{
    io::{BufReader, Cursor},
    path::Path,
};

pub async fn load_string(file_name: &str, file_path: &str) -> anyhow::Result<String> {
    let txt = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join(file_path)
            .join(file_name);
        std::fs::read_to_string(path)?
    };

    Ok(txt)
}

pub async fn load_binary(file_name: &str, file_path: &str) -> anyhow::Result<Vec<u8>> {
    let data = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join(file_path)
            .join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}

pub fn load_texture(
    file: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<Texture> {
    let path = Path::new(file);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let base_path = path.parent().unwrap().to_str().unwrap();

    let data = pollster::block_on(async { load_binary(file_name, base_path).await })?;
    Texture::from_bytes(device, queue, &data, file)
}

pub async fn load_model(
    file: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let path = Path::new(file);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let base_path = path.parent().unwrap().to_str().unwrap();
    let obj_text = pollster::block_on(async { load_string(file_name, base_path).await })?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);
    let (models, obj_materials) = tobj::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| {
            let p = p.to_str().unwrap();

            let mat_text = pollster::block_on(async { load_string(p, base_path).await })
                .expect("failed to load string for model");
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )?;

    let mut materials = vec![];

    for m in obj_materials? {
        let full_texture_path = path.parent().unwrap().join(&m.diffuse_texture);
        let texture_path = full_texture_path.to_str().unwrap();
        let diffuse_texture = load_texture(texture_path, device, queue)?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("resource_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });
        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        });
    }
    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [0.0, 0.0, 0.0],
                        }
                    } else {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            model::Mesh {
                name: file.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}
