// render3d.rs — 3D mesh renderer on WGPU/Vulkan
// Auto-rotating camera + diffuse lighting

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::mesh3d::MeshData;

const SHADER_3D: &str = r#"
struct Uniforms {
    mvp: mat4x4<f32>,
    model: mat4x4<f32>,
    light_dir: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tint: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tint: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = u.mvp * vec4(in.position, 1.0);
    out.world_normal = (u.model * vec4(in.normal, 0.0)).xyz;

    // Gate nodes glow (normal.x > 90 = gate marker flag)
    let is_gate = in.normal.x > 90.0;
    if is_gate {
        // Gate color encoded in normal: y = R*10, z = G*10
        out.color = vec3(in.normal.y / 10.0, in.normal.z / 10.0, 0.3);
    } else {
        // Kenshi desert palette — warm sandstone
        let n_abs = abs(normalize(in.normal));
        out.color = vec3(0.65 + n_abs.x * 0.2, 0.50 + n_abs.y * 0.15, 0.35 + n_abs.z * 0.1);
    }
    out.tint = in.tint;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // A tinted mesh carries its own color: alpha 2 = plain (wires, ground), alpha 1 = lit
    if in.tint.a > 1.5 {
        return vec4(in.tint.rgb, 1.0);
    }
    let n = normalize(in.world_normal);
    if in.tint.a > 0.5 {
        let lit = max(dot(n, normalize(u.light_dir.xyz)), 0.0) * 0.6
                + max(dot(n, normalize(vec3(-0.3, 0.8, -0.5))), 0.0) * 0.25;
        return vec4(in.tint.rgb * (0.35 + lit), 1.0);
    }
    // Gate nodes: emit light (no shading)
    let is_emissive = length(in.world_normal) < 0.01;
    if is_emissive {
        return vec4(in.color * 1.5, 1.0);
    }
    // Three-light rig: key + fill + rim
    let key_dir = normalize(u.light_dir.xyz);
    let fill_dir = normalize(vec3(-0.3, 0.8, -0.5));
    let key = max(dot(n, key_dir), 0.0) * 0.6;
    let fill = max(dot(n, fill_dir), 0.0) * 0.25;
    let ambient = 0.3;
    let rim = pow(1.0 - max(dot(n, normalize(vec3(0.0, 0.3, 1.0))), 0.0), 2.5) * 0.2;
    let base = in.color * (ambient + key + fill) + vec3(rim * 0.8, rim * 0.6, rim * 0.4);
    // Subtle geodesic glow on the background
    return vec4(base, 1.0);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex3D {
    position: [f32; 3],
    normal: [f32; 3],
    tint: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    light_dir: [f32; 4],
}

fn mat4_identity() -> [[f32; 4]; 4] {
    [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]]
}

fn mat4_perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    // WGPU clip space: Z maps to [0, 1] (not OpenGL's [-1, 1])
    let f = 1.0 / (fov_y / 2.0).tan();
    let nf = 1.0 / (near - far);
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far * nf, -1.0],
        [0.0, 0.0, near * far * nf, 0.0],
    ]
}

fn mat4_look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize3(sub3(center, eye));
    let s = normalize3(cross3(f, up));
    let u = cross3(s, f);
    [[s[0],u[0],-f[0],0.0],[s[1],u[1],-f[1],0.0],[s[2],u[2],-f[2],0.0],[-dot3(s,eye),-dot3(u,eye),dot3(f,eye),1.0]]
}

fn mat4_rotate_y(angle: f32) -> [[f32; 4]; 4] {
    let c = angle.cos(); let s = angle.sin();
    [[c,0.0,s,0.0],[0.0,1.0,0.0,0.0],[-s,0.0,c,0.0],[0.0,0.0,0.0,1.0]]
}

fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    // Column-major: m[col][row]. C = A * B means C_col_j = A * B_col_j
    let mut r = [[0.0f32; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            for k in 0..4 {
                r[col][row] += a[k][row] * b[col][k];
            }
        }
    }
    r
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]] }
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }
fn normalize3(v: [f32; 3]) -> [f32; 3] { let l = dot3(v,v).sqrt(); if l < 1e-10 { v } else { [v[0]/l, v[1]/l, v[2]/l] } }

/// A mesh with where it stands (offset) and how big it is (scale).
pub type Placed = (MeshData, [f32; 3], f32);

/// How a scene is laid out when it is uploaded.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    /// Every mesh is fitted to its scale around its offset, and the six gate nodes orbit the lot
    /// (the snow globe, the bestiary).
    Constellation,
    /// The meshes are already in world units: the scale multiplies, the offset moves, nothing is
    /// added and nothing is fitted (the network map).
    World,
}

/// Tint alpha of a mesh that takes the scene's light.
pub const TINT_LIT: f32 = 1.0;
/// Tint alpha of a mesh drawn in its plain color (wires, ground).
pub const TINT_FLAT: f32 = 2.0;
/// How fast the camera goes round, in radians a second.
const ORBIT_SPEED: f32 = 0.5;

/// How far out the camera goes round, in scene radii: a world fills the frame, a constellation
/// keeps room for the nodes that orbit it.
fn orbit_of(layout: Layout) -> f32 {
    match layout {
        Layout::Constellation => 2.5,
        Layout::World => 1.7,
    }
}

/// The camera's matrix `t` seconds into its round about a scene.
fn camera(center: [f32; 3], radius: f32, orbit: f32, aspect: f32, t: f32) -> [[f32; 4]; 4] {
    let dist = radius * orbit;
    let angle = t * ORBIT_SPEED;
    let eye = [
        center[0] + dist * angle.cos(),
        center[1] + radius * 0.5,
        center[2] + dist * angle.sin(),
    ];
    let look = mat4_look_at(eye, center, [0.0, 1.0, 0.0]);
    let proj = mat4_perspective(std::f32::consts::FRAC_PI_4, aspect, 0.1, dist * 10.0);
    mat4_mul(proj, look)
}

/// Merge the meshes into one vertex list and one index list. `tints` has one color per mesh
/// (alpha 0, or no entry: the mesh keeps the engine's own palette). Returns the scene's
/// center and radius with them.
fn build_scene(meshes: &[Placed], tints: &[[f32; 4]], layout: Layout) -> (Vec<Vertex3D>, Vec<u32>, [f32; 3], f32) {
    // Merge all meshes into one mega-buffer, each offset to its constellation position
    let mut vertices: Vec<Vertex3D> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut scene_radius: f32 = 0.0;
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];

    for (m, (mesh, offset, scale)) in meshes.iter().enumerate() {
        let tint = tints.get(m).copied().unwrap_or([0.0; 4]);
        let base_vert = vertices.len() as u32;
        // Compute per-mesh bounding box for normalization
        let mut mmin = [f32::MAX; 3];
        let mut mmax = [f32::MIN; 3];
        for p in &mesh.positions {
            for i in 0..3 { mmin[i] = mmin[i].min(p[i]); mmax[i] = mmax[i].max(p[i]); }
        }
        let (mcenter, s) = match layout {
            Layout::Constellation => {
                let mradius = sub3(mmax, mmin).iter().map(|d| d*d).sum::<f32>().sqrt() / 2.0;
                ([(mmin[0]+mmax[0])/2.0, (mmin[1]+mmax[1])/2.0, (mmin[2]+mmax[2])/2.0],
                 if mradius > 0.001 { scale / mradius } else { 1.0 })
            }
            Layout::World => ([0.0; 3], *scale),
        };

        for (i, p) in mesh.positions.iter().enumerate() {
            let n = if i < mesh.normals.len() { mesh.normals[i] } else { [0.0, 1.0, 0.0] };
            let position = [
                (p[0] - mcenter[0]) * s + offset[0],
                (p[1] - mcenter[1]) * s + offset[1],
                (p[2] - mcenter[2]) * s + offset[2],
            ];
            for i in 0..3 { lo[i] = lo[i].min(position[i]); hi[i] = hi[i].max(position[i]); }
            vertices.push(Vertex3D { position, normal: n, tint });
            let dist = ((offset[0]*offset[0]) + (offset[1]*offset[1]) + (offset[2]*offset[2])).sqrt() + scale;
            if dist > scene_radius { scene_radius = dist; }
        }
        for idx in &mesh.indices {
            indices.push(base_vert + idx);
        }
    }

    if layout == Layout::World {
        // A world is looked at from around the middle of what is in it.
        if vertices.is_empty() { return (vertices, indices, [0.0; 3], 10.0); }
        let center = [(lo[0]+hi[0])/2.0, (lo[1]+hi[1])/2.0, (lo[2]+hi[2])/2.0];
        let radius = (sub3(hi, lo).iter().map(|d| d*d).sum::<f32>().sqrt() / 2.0).max(1.0);
        return (vertices, indices, center, radius);
    }

    let center = [0.0f32; 3];
    let radius = if scene_radius > 0.1 { scene_radius } else { 10.0 };
    println!("  Bestiary scene: {} meshes, {} total verts, {} total tris, radius={:.1}",
        meshes.len(), vertices.len(), indices.len() / 3, radius);

    // Gate nodes — octahedrons at the edge of the constellation
    let gate_orbit = radius * 1.1;
    let node_size = radius * 0.015;
    struct GateNode { angle: f32, elev: f32, r: f32, g: f32 }
    let gates = vec![
        GateNode { angle: 0.0,    elev: 0.2,  r: 0.0, g: 8.0 },  // northGate — green
        GateNode { angle: 1.05,   elev: 0.0,  r: 8.0, g: 4.0 },  // eastGate — orange
        GateNode { angle: 2.09,   elev: -0.1, r: 4.0, g: 8.0 },  // ironGate — lime
        GateNode { angle: 3.14,   elev: 0.1,  r: 2.0, g: 6.0 },  // sporeGate — teal
        GateNode { angle: 4.19,   elev: 0.3,  r: 8.0, g: 0.0 },  // grapheneGate — red
        GateNode { angle: 5.24,   elev: -0.2, r: 3.0, g: 3.0 },  // graftGate — dim
    ];
    for g in &gates {
        let cx = gate_orbit * g.angle.cos() * g.elev.cos();
        let cy = gate_orbit * g.elev.sin();
        let cz = gate_orbit * g.angle.sin() * g.elev.cos();
        let s = node_size;
        let base = vertices.len() as u32;
        let gn = [99.0, g.r, g.g];
        let none = [0.0; 4];
        vertices.push(Vertex3D { position: [cx+s, cy, cz], normal: gn, tint: none });
        vertices.push(Vertex3D { position: [cx-s, cy, cz], normal: gn, tint: none });
        vertices.push(Vertex3D { position: [cx, cy+s, cz], normal: gn, tint: none });
        vertices.push(Vertex3D { position: [cx, cy-s, cz], normal: gn, tint: none });
        vertices.push(Vertex3D { position: [cx, cy, cz+s], normal: gn, tint: none });
        vertices.push(Vertex3D { position: [cx, cy, cz-s], normal: gn, tint: none });
        for &[a,b,c] in &[[0,2,4],[2,1,4],[1,3,4],[3,0,4],[2,0,5],[1,2,5],[3,1,5],[0,3,5u32]] {
            indices.push(base+a); indices.push(base+b); indices.push(base+c);
        }
    }
    println!("  + {} gate nodes orbiting at {:.1}", gates.len(), gate_orbit);

    (vertices, indices, center, radius)
}

/// What draws a scene into a color view: the window uses it, and so does the snapshot.
struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    index_count: u32,
    mesh_center: [f32; 3],
    mesh_radius: f32,
    orbit: f32,
}

impl Renderer {
    fn new(device: wgpu::Device, queue: wgpu::Queue, format: wgpu::TextureFormat,
           meshes: &[Placed], tints: &[[f32; 4]], layout: Layout) -> Self {
        let (vertices, indices, center, radius) = build_scene(meshes, tints, layout);
        let (vertex_buf, index_buf) = Self::upload(&device, &vertices, &indices);

        let uniforms = Uniforms { mvp: mat4_identity(), model: mat4_identity(), light_dir: [0.5, 1.0, 0.3, 0.0] };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniforms"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader3d"), source: wgpu::ShaderSource::Wgsl(SHADER_3D.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pl"), bind_group_layouts: &[&bind_group_layout], push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rp3d"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex3D>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 24, shader_location: 2 },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None, // Show both sides
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        println!("  Uploaded: {} verts + {} indices to GPU", vertices.len(), indices.len());

        Self {
            device, queue, pipeline, vertex_buf, index_buf, uniform_buf, bind_group,
            index_count: indices.len() as u32,
            mesh_center: center, mesh_radius: radius, orbit: orbit_of(layout),
        }
    }

    fn upload(device: &wgpu::Device, vertices: &[Vertex3D], indices: &[u32]) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        (vertex_buf, index_buf)
    }

    /// Put another scene in place of the one being drawn.
    fn set_scene(&mut self, meshes: &[Placed], tints: &[[f32; 4]], layout: Layout) {
        let (vertices, indices, center, radius) = build_scene(meshes, tints, layout);
        let (vertex_buf, index_buf) = Self::upload(&self.device, &vertices, &indices);
        self.vertex_buf = vertex_buf;
        self.index_buf = index_buf;
        self.index_count = indices.len() as u32;
        self.mesh_center = center;
        self.mesh_radius = radius;
        self.orbit = orbit_of(layout);
    }

    /// Draw the scene as the orbiting camera sees it `t` seconds into its round.
    fn draw(&self, view: &wgpu::TextureView, depth_view: &wgpu::TextureView, aspect: f32, t: f32) {
        let model = mat4_identity();
        let mvp = mat4_mul(camera(self.mesh_center, self.mesh_radius, self.orbit, aspect, t), model);

        let uniforms = Uniforms {
            mvp,
            model,
            light_dir: [0.5, 1.0, 0.3, 0.0],
        };
        self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[uniforms]));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rp"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.02, b: 0.05, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.index_count, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

fn create_depth(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }).create_view(&wgpu::TextureViewDescriptor::default())
}

pub struct Gpu3D {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    depth_view: wgpu::TextureView,
    start_time: Instant,
    gpu_name: String,
    r: Renderer,
}

impl Gpu3D {
    fn new(window: Arc<Window>, meshes: &[Placed], tints: &[[f32; 4]], layout: Layout) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12,
            ..Default::default()
        });
        let surface = instance.create_surface(window).expect("surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).expect("adapter");

        let info = adapter.get_info();
        let gpu_name = info.name.clone();
        println!("  GPU: {} ({:?})", info.name, info.backend);

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("KenshiReborn3D"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )).expect("device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth_view = create_depth(&device, config.width, config.height);
        let r = Renderer::new(device, queue, format, meshes, tints, layout);

        Self { surface, config, depth_view, start_time: Instant::now(), gpu_name, r }
    }

    fn resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.r.device, &self.config);
            self.depth_view = create_depth(&self.r.device, w, h);
        }
    }

    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let t = self.start_time.elapsed().as_secs_f32();
        let aspect = self.config.width as f32 / self.config.height.max(1) as f32;

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.r.draw(&view, &self.depth_view, aspect, t);
        output.present();
        Ok(())
    }
}

/// Where points of a scene land in a snapshot of it: fractions of the picture's width and
/// height from its top left corner, or None for a point behind the camera. The scene, layout,
/// size and angle are the ones given to `snapshot`.
pub fn project(meshes: &[Placed], tints: &[[f32; 4]], layout: Layout,
               width: u32, height: u32, angle: f32, points: &[[f32; 3]]) -> Vec<Option<[f32; 2]>> {
    let (_, _, center, radius) = build_scene(meshes, tints, layout);
    let m = camera(center, radius, orbit_of(layout), width as f32 / height.max(1) as f32, angle / ORBIT_SPEED);
    points.iter().map(|p| {
        let clip = |row: usize| m[0][row] * p[0] + m[1][row] * p[1] + m[2][row] * p[2] + m[3][row];
        let w = clip(3);
        (w > 1e-6).then(|| [(clip(0) / w + 1.0) / 2.0, (1.0 - clip(1) / w) / 2.0])
    }).collect()
}

/// Draw a scene once, without a window, and write it as a PNG. `angle` is where the camera
/// stands on its round, in radians. Returns the name of the GPU that drew it.
pub fn snapshot(meshes: &[Placed], tints: &[[f32; 4]], layout: Layout,
                width: u32, height: u32, angle: f32, path: &str) -> Result<String, String> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12,
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })).ok_or("no GPU to draw with")?;
    let gpu_name = adapter.get_info().name;

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("KenshiRebornSnapshot"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
        },
        None,
    )).map_err(|e| e.to_string())?;

    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let extent = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("snapshot"),
        size: extent,
        mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = create_depth(&device, width, height);

    let r = Renderer::new(device, queue, format, meshes, tints, layout);
    r.draw(&view, &depth_view, width as f32 / height.max(1) as f32, angle / ORBIT_SPEED);

    // Rows come back padded to the GPU's row alignment.
    let row = width * 4;
    let padded = row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let out = r.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("snapshot-out"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = r.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("snapshot-read") });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo {
            buffer: &out,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(height) },
        },
        extent,
    );
    r.queue.submit(std::iter::once(encoder.finish()));

    let slice = out.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| { let _ = tx.send(result); });
    r.device.poll(wgpu::Maintain::Wait);
    rx.recv().map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;

    let mut pixels = Vec::with_capacity((row * height) as usize);
    {
        let data = slice.get_mapped_range();
        for y in 0..height {
            let start = (y * padded) as usize;
            pixels.extend_from_slice(&data[start..start + row as usize]);
        }
    }
    out.unmap();

    image::RgbaImage::from_raw(width, height, pixels)
        .ok_or("the picture came back the wrong size")?
        .save(path)
        .map_err(|e| e.to_string())?;
    Ok(gpu_name)
}

/// The next state of a scene that changes while its window is open.
pub struct SceneUpdate {
    pub meshes: Vec<Placed>,
    pub tints: Vec<[f32; 4]>,
    pub title: String,
}

pub struct App3D {
    pub gpu: Option<Gpu3D>,
    pub window: Option<Arc<Window>>,
    pub meshes: Vec<(MeshData, [f32; 3], f32)>,
    pub title: String,
    pub frame_count: u64,
    pub start: Instant,
    /// One tint per mesh; empty when the meshes keep the engine's own palette.
    pub tints: Vec<[f32; 4]>,
    pub layout: Layout,
    /// A scene that changes while its window is open sends its next state here.
    pub updates: Option<std::sync::mpsc::Receiver<SceneUpdate>>,
}

impl App3D {
    pub fn new_single(mesh: MeshData) -> Self {
        let title = format!("KenshiReborn v0.7.0 — {} — Snow Globe · RTX 5090", mesh.name);
        let meshes = vec![(mesh, [0.0, 0.0, 0.0], 1.0)];
        Self { gpu: None, window: None, meshes, title, frame_count: 0, start: Instant::now(),
               tints: Vec::new(), layout: Layout::Constellation, updates: None }
    }
    pub fn new_bestiary(meshes: Vec<(MeshData, [f32; 3], f32)>, count: usize) -> Self {
        let title = format!("KenshiReborn v0.7.0 — BESTIARY × {} — Snow Globe · RTX 5090", count);
        Self { gpu: None, window: None, meshes, title, frame_count: 0, start: Instant::now(),
               tints: Vec::new(), layout: Layout::Constellation, updates: None }
    }
}

impl ApplicationHandler for App3D {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        let attrs = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 720u32));
        let window = Arc::new(event_loop.create_window(attrs).expect("window"));
        let gpu = Gpu3D::new(window.clone(), &self.meshes, &self.tints, self.layout);
        println!("  Pipeline: 3D render with depth + lighting");
        if self.layout == Layout::Constellation {
            println!("  Camera: auto-orbit around bestiary constellation");
            println!("  STATUS: ██████████ BESTIARY RENDERING LIVE\n");
        } else {
            println!("  Camera: auto-orbit around the world\n");
        }
        self.gpu = Some(gpu);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                let fps = self.frame_count as f64 / self.start.elapsed().as_secs_f64();
                println!("\n◆ CLOSED — {} frames, {:.1} FPS", self.frame_count, fps);
                event_loop.exit();
            }
            WindowEvent::Resized(s) => { if let Some(g) = &mut self.gpu { g.resize(s.width, s.height); } }
            WindowEvent::RedrawRequested => {
                // A scene that follows something outside: take its newest state, if one came.
                let next = self.updates.as_ref().and_then(|rx| rx.try_iter().last());
                if let Some(update) = next {
                    self.meshes = update.meshes;
                    self.tints = update.tints;
                    self.title = update.title;
                    if let Some(g) = &mut self.gpu { g.r.set_scene(&self.meshes, &self.tints, self.layout); }
                    if let Some(w) = &self.window { w.set_title(&self.title); }
                }
                if let Some(g) = &self.gpu {
                    match g.render() {
                        Ok(()) => self.frame_count += 1,
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let (w,h) = (g.config.width, g.config.height);
                            if let Some(g) = &mut self.gpu { g.resize(w,h); }
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("render: {:?}", e),
                    }
                }
                if let Some(w) = &self.window { w.request_redraw(); }
            }
            _ => {}
        }
    }
}

pub fn run_mesh_render(mesh_path: &str) {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  KENSHI REBORN v0.7.0 — WOOD AGE                      ║");
    println!("║  3D Mesh Renderer · Vulkan · RTX 5090                  ║");
    println!("║  The first Kenshi mesh rendered by a sovereign engine   ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("◆ LOADING MESH: {}", mesh_path);
    let mesh = load_mesh(mesh_path).unwrap_or_else(|| panic!("Failed to load mesh: {}", mesh_path));
    println!("  ✓ {} vertices, {} triangles\n", mesh.positions.len(), mesh.indices.len() / 3);

    println!("◆ INITIALIZING 3D RENDERER");
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App3D::new_single(mesh);
    event_loop.run_app(&mut app).expect("run");
}

pub fn run_bestiary(mesh_dir: &str) {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  KENSHI REBORN v0.7.0 — WORLD MAP BESTIARY            ║");
    println!("║  Creatures pinned to biomes · 8×8 world grid           ║");
    println!("║  RTX 5090 · Vulkan · Sovereign engine                  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Biome → creature mapping from Kenshi game data
    // Grid coords: (col, row) on 8×8 map, (0,0)=top-left, (7,7)=bottom-right
    // World space: X = col * TILE, Z = row * TILE (Y = altitude)
    struct BiomePin { name: &'static str, gx: f32, gz: f32, creatures: &'static [&'static str], color: [f32;3] }
    let biomes = vec![
        BiomePin { name: "Venge",           gx: 4.5, gz: 5.0, creatures: &["Mammal_beak", "land_bat", "bird"],
            color: [0.8, 0.3, 0.15] },
        BiomePin { name: "Deadlands",       gx: 3.5, gz: 3.0, creatures: &["brokendrone", "robot_worker"],
            color: [0.4, 0.4, 0.4] },
        BiomePin { name: "Great Desert",    gx: 5.0, gz: 1.5, creatures: &["antilop250", "goat", "bull"],
            color: [0.9, 0.82, 0.5] },
        BiomePin { name: "Swamp",           gx: 2.5, gz: 5.5, creatures: &["swamp_duck", "frog_gorilla"],
            color: [0.3, 0.5, 0.2] },
        BiomePin { name: "Fog Islands",     gx: 1.5, gz: 2.5, creatures: &["flesh_insect"],
            color: [0.6, 0.7, 0.6] },
        BiomePin { name: "Gut",             gx: 6.5, gz: 3.5, creatures: &["elephant_turtle", "Crab"],
            color: [0.65, 0.55, 0.4] },
        BiomePin { name: "Leviathan Coast", gx: 1.5, gz: 0.0, creatures: &["whale2", "Big-Bones"],
            color: [0.4, 0.6, 0.8] },
        BiomePin { name: "Border Zone",     gx: 3.0, gz: 3.5, creatures: &["dog"],
            color: [0.7, 0.6, 0.45] },
        BiomePin { name: "Howler Maze",     gx: 5.5, gz: 2.5, creatures: &["skimmer"],
            color: [0.8, 0.6, 0.3] },
        BiomePin { name: "Ashlands",        gx: 6.5, gz: 6.5, creatures: &["robot_worker"],
            color: [0.25, 0.12, 0.12] },
        BiomePin { name: "Spider Plains",   gx: 0.5, gz: 4.5, creatures: &["cage_beast"],
            color: [0.55, 0.4, 0.47] },
        BiomePin { name: "Cannibal Plains", gx: 1.5, gz: 1.0, creatures: &["pack_beast"],
            color: [0.6, 0.6, 0.6] },
    ];

    let mesh_path = std::path::Path::new(mesh_dir);
    let mut entries: Vec<_> = std::fs::read_dir(mesh_path)
        .expect("read mesh dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "mesh"))
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().map(|m| m.len()).unwrap_or(0)));

    let skip = ["_fur", "fur.", "bags", "luggage", "spikes", "hair", "nohorns", "nospikes",
                 "polySurface", "dog1"];

    // Load all meshes into a name→MeshData map
    let mut mesh_map: std::collections::HashMap<String, MeshData> = std::collections::HashMap::new();
    let mut failed = 0u32;

    println!("◆ LOADING BESTIARY from {}\n", mesh_dir);
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.iter().any(|s| name.contains(s)) { continue; }
        let path_str = entry.path().to_string_lossy().to_string();
        match load_mesh(&path_str) {
            Some(m) => {
                println!("  ✓ {} — {} verts, {} tris", m.name, m.positions.len(), m.indices.len() / 3);
                let key = m.name.clone();
                mesh_map.insert(key, m);
            }
            None => { println!("  ✗ {} — parse failed", name); failed += 1; }
        }
    }
    println!("\n  Loaded {} unique meshes ({} failed)\n", mesh_map.len(), failed);

    // Place creatures at their biome world-map positions
    // World tile = 6.0 units, so 8×8 grid → 48×48 world space
    let tile = 6.0f32;
    let creature_scale = 1.2f32;
    let mut placed: Vec<(MeshData, [f32; 3], f32)> = Vec::new();

    println!("◆ PINNING TO WORLD MAP (8×8 grid, {} units/tile)\n", tile);
    for biome in &biomes {
        let bx = (biome.gx - 3.5) * tile;  // center the 8×8 grid
        let bz = (biome.gz - 3.5) * tile;
        let mut slot = 0;
        for creature_name in biome.creatures {
            if let Some(mesh) = mesh_map.remove(*creature_name) {
                let spread = slot as f32 * 1.5;
                let offset = [bx + spread, 0.0, bz];
                println!("  📍 {} → {} at ({:.1}, {:.1})", creature_name, biome.name, bx + spread, bz);
                placed.push((mesh, offset, creature_scale));
                slot += 1;
            } else {
                println!("  ⚠  {} not found for {}", creature_name, biome.name);
            }
        }
    }

    // Any remaining un-pinned meshes go in a "wilds" orbit ring
    let remaining: Vec<_> = mesh_map.into_iter().collect();
    if !remaining.is_empty() {
        println!("\n  {} unplaced creatures → orbiting in the Wilds ring", remaining.len());
        let wilds_r = 4.5 * tile;
        let golden = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
        for (i, (name, mesh)) in remaining.into_iter().enumerate() {
            let angle = i as f32 * golden;
            let offset = [wilds_r * angle.cos(), 2.0, wilds_r * angle.sin()];
            println!("  🌀 {} → Wilds orbit", name);
            placed.push((mesh, offset, creature_scale * 0.8));
        }
    }

    let count = placed.len();
    println!("\n◆ TOTAL: {} creatures pinned to {} biomes on the world map", count, biomes.len());
    println!("◆ INITIALIZING WORLD MAP RENDERER\n");

    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App3D::new_bestiary(placed, count);
    event_loop.run_app(&mut app).expect("run");
}

use crate::mesh3d::load_mesh;
use wgpu::util::DeviceExt;

#[cfg(test)]
mod tests {
    use super::*;

    fn body(at: [f32; 3]) -> Placed {
        (MeshData { name: "body".into(), positions: vec![[-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]], normals: vec![], indices: vec![] }, at, 1.0)
    }

    #[test]
    fn the_middle_of_a_world_lands_in_the_middle_of_its_picture() {
        // Two bodies spanning -5..7, -1..3, -1..1: the middle is [1, 1, 0].
        let scene = [body([-4.0, 0.0, 0.0]), body([6.0, 2.0, 0.0])];
        let at = project(&scene, &[], Layout::World, 1600, 900, 2.0, &[[1.0, 1.0, 0.0], [1.0, 3.0, 0.0]]);
        let middle = at[0].unwrap();
        assert!((middle[0] - 0.5).abs() < 1e-4 && (middle[1] - 0.5).abs() < 1e-4, "{middle:?}");
        // A point straight above the middle is straight above it in the picture.
        let above = at[1].unwrap();
        assert!((above[0] - 0.5).abs() < 1e-4 && above[1] < 0.5, "{above:?}");
    }

    #[test]
    fn a_point_behind_the_camera_has_no_place_in_the_picture() {
        let scene = [body([-4.0, 0.0, 0.0]), body([6.0, 2.0, 0.0])];
        // At angle 0 the camera stands out along +X and looks back at the middle.
        let at = project(&scene, &[], Layout::World, 1600, 900, 0.0, &[[500.0, 1.0, 0.0]]);
        assert!(at[0].is_none());
    }
}
