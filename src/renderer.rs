use wgpu::wgc::device;

use crate::renderer::math::{Camera, CameraUniforms};

use {
    bytemuck::{Pod, Zeroable}, std::panic::set_hook, wgpu::{self, PipelineCompilationOptions, PipelineLayout, wgc::binding_model::CreatePipelineLayoutError}
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
mod math;


pub struct PathTracer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    bind_group1: wgpu::BindGroup,
    bind_group2: wgpu::BindGroup,
    frame_count: u32,
    last_fps_instant: Instant,
    pub camera: math::Camera,
    pub fov: f32,
}


#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    width: u32,
    height: u32,
    time:f32,
    _pad: f32,
    camera: math::Camera,
    _pad2:f32
}

impl PathTracer {
    fn create_sample_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::TextureDescriptor, wgpu::Texture, wgpu::Texture) {
    let desc = wgpu::TextureDescriptor {
        label: Some("radiance samples"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    };
    let a1 = device.create_texture(&desc);
    let a2 = device.create_texture(&desc);
    (desc, a1, a2)
    }


    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> PathTracer {
        device.on_uncaptured_error(Box::new(|error| {
            panic!("Aborting due to an error: {}", error);
        }));

        let shader_module = compile_shader_module((&device));
        let pe = create_pipeline_layout(&device, &shader_module);
        let layout = pe.1;
        let pipeline = pe.0;
         let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let mils = duration.as_secs_f32();
        let mut cams = math::Camera::new(math::Vec4::new(4.0,0.0,-1.0, 0.0));
        let uniforms = Uniforms {
            width: 1920,
            height: 1200,
            time: 0.0,
            _pad: 1.0,
            camera:cams,
            _pad2: 10.0
        };
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let storage_texture1 = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Storage Texture"),
            size: wgpu::Extent3d {
                width: 1920,
                height: 1200,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let storage_texture_view1 = storage_texture1.create_view(&wgpu::TextureViewDescriptor::default());

        let storage_texture2 = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Storage Texture"),
            size: wgpu::Extent3d {
                width: 1920,
                height: 1200,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let storage_texture_view2 = storage_texture2.create_view(&wgpu::TextureViewDescriptor::default());



        let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
                
            }, 
            wgpu::BindGroupEntry{
                binding: 1, 
                resource:wgpu::BindingResource::TextureView(&storage_texture_view1)
            },
            wgpu::BindGroupEntry{
                binding: 2, 
                resource:wgpu::BindingResource::TextureView(&storage_texture_view2)
            }],
        });

        let bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
                
            }, 
            wgpu::BindGroupEntry{
                binding: 1, 
                resource:wgpu::BindingResource::TextureView(&storage_texture_view2)
            },
            wgpu::BindGroupEntry{
                binding: 2, 
                resource:wgpu::BindingResource::TextureView(&storage_texture_view1)
            }],
        });

        let camera = math::Camera::look_at(
            math::Vec4::new(0., -0.0, 0.0,0.0),
            math::Vec4::new(0., -0.0, -3.0,0.0),
            math::Vec4::new(0., 1., 0.0,0.0),
        );
        let fov = 10.0;

        // TODO: initialize GPU resources

        PathTracer {
            device,
            queue,
            pipeline,
            uniforms,
            uniform_buffer,
            bind_group1,
            bind_group2,
            frame_count:0,
            last_fps_instant:Instant::now(),
            camera,
            fov
        }
    }
    pub fn render_frame(&mut self, target: &wgpu::TextureView) {
        
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render frame"),
            });
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).unwrap();
        let uniforms = Uniforms {
            width: 1920,
            height: 1200,
            time: self.fov,
            _pad: self.frame_count as f32,
            camera: self.camera,
            _pad2: self.fov,
        };
        self.queue.write_buffer(
        &self.uniform_buffer,
        0,
        bytemuck::bytes_of(&uniforms),
    );
        self.frame_count += 1;
        self.frame_count %= 1000000;


        
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("path tracer render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        
        render_pass.set_pipeline(&self.pipeline);
        if(self.frame_count%2 == 1){
            render_pass.set_bind_group(0, &self.bind_group1, &[]);
        }
        else{
            render_pass.set_bind_group(0, &self.bind_group2, &[]);
        }
        
        
        // Draw 1 instance of a polygon with 3 vertices.
        render_pass.draw(0..6, 0..1);

        // End the render pass by consuming the object.
        drop(render_pass);

        let command_buffer = encoder.finish();
        self.queue.submit(Some(command_buffer));
    }
}

fn compile_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    use std::borrow::Cow;

    let code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders.wgsl"));
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
    })
}


fn create_pipeline_layout(
    device: &wgpu::Device,
    shader_module: &wgpu::ShaderModule,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm, 
                        view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
                }
            ],
        });
    


        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Pipeline Layout"),
    bind_group_layouts: &[&bind_group_layout],
    push_constant_ranges: &[],
    });
    let pipeline: wgpu::RenderPipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("path tracer"),
        layout: Some(&pipeline_layout),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        },
        vertex: wgpu::VertexState {
            module: shader_module,
            entry_point: Some("path_tracer_vs"),
            buffers: &[],
            compilation_options: PipelineCompilationOptions::default(),
        },
        
        fragment: Some(wgpu::FragmentState {
            module: shader_module,
            entry_point: Some("path_tracer_fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    return (pipeline, bind_group_layout);
}
