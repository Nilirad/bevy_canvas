use std::mem::size_of;

use bevy::{
    asset::{Assets, Handle, HandleUntyped},
    core::AsBytes,
    ecs::world::World,
    reflect::TypeUuid,
    render::{
        camera::ActiveCameras,
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassColorAttachmentDescriptor,
            RenderPassDepthStencilAttachmentDescriptor, TextureAttachment,
        },
        pipeline::{
            BindGroupDescriptor, BindType, BindingDescriptor, BindingShaderStage, BlendFactor,
            BlendOperation, BlendState, ColorTargetState, ColorWrite, CompareFunction, CullMode,
            DepthBiasState, DepthStencilState, FrontFace, IndexFormat, InputStepMode,
            MultisampleState, PipelineDescriptor, PipelineLayout, PolygonMode, PrimitiveState,
            PrimitiveTopology, StencilFaceState, StencilState, UniformProperty, VertexAttribute,
            VertexBufferLayout, VertexFormat,
        },
        render_graph::{
            base, Node, RenderGraph, ResourceSlotInfo, ResourceSlots, SlotLabel,
            WindowSwapChainNode, WindowTextureNode,
        },
        renderer::{
            BindGroup, BufferInfo, BufferUsage, RenderContext, RenderResourceBindings,
            RenderResourceContext, RenderResourceType,
        },
        shader::{Shader, ShaderStage, ShaderStages},
        texture::TextureFormat,
    },
};
use types::Vertex;

use crate::render::types::BufferPair;

pub mod types;

pub mod node {
    pub const CANVAS: &str = "bevy_canvas:render:canvas_node";
}

pub const CANVAS_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 0xae17f73d2a1827d1);
const COLOR_ATTACHMENT_SLOT: SlotLabel = SlotLabel::Index(0);
const DEPTH_STENCIL_ATTACHMENT_SLOT: SlotLabel = SlotLabel::Index(1);
const VERTEX_SHADER_SRC: &str = if cfg!(target_arch = "wasm32") {
    "" // TODO: include_str!("shader.es.vert")
} else {
    include_str!("shader.vert")
};
const FRAGMENT_SHADER_SRC: &str = if cfg!(target_arch = "wasm32") {
    "" // TODO: include_str!("shader.es.frag")
} else {
    include_str!("shader.frag")
};

#[derive(Debug)]
pub struct CanvasNode {
    input_slots: [ResourceSlotInfo; 2],
    vertex_buffers: BufferPair,
}

impl CanvasNode {
    fn new() -> Self {
        let inputs = [
            ResourceSlotInfo::new(
                "bevy_canvas:render:canvas_node:color_attachment",
                RenderResourceType::Texture,
            ),
            ResourceSlotInfo::new(
                "bevy_canvas:render:canvas_node:depth_stencil_attachment",
                RenderResourceType::Texture,
            ),
        ];

        Self {
            input_slots: inputs,
            vertex_buffers: BufferPair::new(),
        }
    }
}

impl Node for CanvasNode {
    fn input(&self) -> &[ResourceSlotInfo] {
        &self.input_slots
    }

    fn prepare(&mut self, world: &mut World) {
        let mut canvas = world.get_resource_mut::<crate::canvas::Canvas>().unwrap();
        // TODO: Try optimizing. Make benchmarks. (use mem::swap?)
        self.vertex_buffers = std::mem::take(&mut canvas.vertex_buffers);
    }

    fn update(
        &mut self,
        world: &World,
        render_context: &mut dyn RenderContext,
        input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        let pipeline = CANVAS_PIPELINE_HANDLE.typed::<PipelineDescriptor>();
        let render_resource_bindings = world.get_resource::<RenderResourceBindings>().unwrap();
        let render_resources = render_context.resources();
        let pipelines = world.get_resource::<Assets<PipelineDescriptor>>().unwrap();

        let active_cameras = world.get_resource::<ActiveCameras>().unwrap();
        let camera_2d = active_cameras.get("Camera2d").unwrap();
        let camera_binding = if let Some(binding) = camera_2d.bindings.get("CameraViewProj") {
            binding.clone()
        } else {
            println!("Cannot find camera!");
            return;
        };

        let camera_bind_group_descriptor = pipelines
            .get(pipeline.clone())
            .unwrap()
            .get_layout()
            .unwrap()
            .get_bind_group(0)
            .unwrap();
        let camera_bind_group_id = if render_context
            .resources()
            .bind_group_descriptor_exists(camera_bind_group_descriptor.id)
        {
            let camera_bind_group = BindGroup::build().add_binding(0, camera_binding).finish();
            render_resources.create_bind_group(camera_bind_group_descriptor.id, &camera_bind_group);
            camera_bind_group.id
        } else {
            println!("Cannot find camera bind group descriptor!");
            return;
        };

        let sample_count = world
            .get_resource::<bevy::render::render_graph::base::Msaa>()
            .map(|msaa| msaa.samples)
            .unwrap_or(1);

        let index_buffer = render_resources.create_buffer_with_data(
            BufferInfo {
                buffer_usage: BufferUsage::INDEX,
                ..Default::default()
            },
            &self.vertex_buffers.indices.as_bytes(),
        );

        let vertex_buffer = render_resources.create_buffer_with_data(
            BufferInfo {
                buffer_usage: BufferUsage::VERTEX,
                ..Default::default()
            },
            &self.vertex_buffers.vertices.as_bytes(),
        );

        let pass_descriptor = pass_descriptor(input, sample_count);
        render_context.begin_pass(&pass_descriptor, &render_resource_bindings, &mut |pass| {
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(
                0,
                camera_bind_group_descriptor.id,
                camera_bind_group_id,
                None,
            );
            pass.set_vertex_buffer(0, vertex_buffer, 0);
            pass.set_index_buffer(index_buffer, 0, IndexFormat::Uint32);
            pass.draw_indexed(0..(self.vertex_buffers.indices.len() as u32), 0, 0..1);
        });

        let render_resources = render_context.resources();
        render_resources.remove_buffer(vertex_buffer);
        render_resources.remove_buffer(index_buffer);
    }
}

pub(crate) fn setup_canvas_node(world: &mut World) {
    let world = world.cell();
    let mut pipelines = world
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();
    let mut shaders = world.get_resource_mut::<Assets<Shader>>().unwrap();
    let mut render_graph = world.get_resource_mut::<RenderGraph>().unwrap();
    let render_resource_context = world
        .get_resource::<Box<dyn RenderResourceContext>>()
        .unwrap();

    pipelines.set_untracked(CANVAS_PIPELINE_HANDLE, pipeline_descriptor(&mut *shaders));

    let pipeline_handle: Handle<PipelineDescriptor> = CANVAS_PIPELINE_HANDLE.typed();
    let pipeline_descriptor = pipelines.get(pipeline_handle.clone()).unwrap();
    render_resource_context.create_render_pipeline(pipeline_handle, pipeline_descriptor, &*shaders);

    render_graph.add_node(node::CANVAS, CanvasNode::new());
    render_graph
        .add_slot_edge(
            base::node::PRIMARY_SWAP_CHAIN,
            WindowSwapChainNode::OUT_TEXTURE,
            node::CANVAS,
            COLOR_ATTACHMENT_SLOT,
        )
        .unwrap();

    render_graph
        .add_slot_edge(
            base::node::MAIN_DEPTH_TEXTURE,
            WindowTextureNode::OUT_TEXTURE,
            node::CANVAS,
            DEPTH_STENCIL_ATTACHMENT_SLOT,
        )
        .unwrap();

    render_graph
        .add_node_edge(base::node::MAIN_PASS, node::CANVAS)
        .unwrap();
}

fn pass_descriptor(input: &ResourceSlots, sample_count: u32) -> PassDescriptor {
    let color_texture = input
        .get(COLOR_ATTACHMENT_SLOT)
        .unwrap()
        .get_texture()
        .unwrap();

    let depth_stencil_texture = input
        .get(DEPTH_STENCIL_ATTACHMENT_SLOT)
        .unwrap()
        .get_texture()
        .unwrap();

    PassDescriptor {
        color_attachments: vec![RenderPassColorAttachmentDescriptor {
            attachment: TextureAttachment::Id(color_texture),
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Load,
                store: true,
            },
        }],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
            attachment: TextureAttachment::Id(depth_stencil_texture),
            depth_ops: Some(Operations {
                load: LoadOp::Load,
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count,
    }
}

fn pipeline_descriptor(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    // BUG: Setting a multisample state with more than 1 sample causes a
    // validation error even if MSAA is set to many samples.

    // TODO: Remove this panic after implementing WebGL support!
    if cfg!(target_arch = "wasm32") {
        panic!("Currently bevy_canvas does not support WebGL shaders. Feel free to submit a PR :)");
    }

    PipelineDescriptor {
        name: Some("CanvasPipeline".into()),
        layout: Some(PipelineLayout {
            bind_groups: vec![BindGroupDescriptor::new(
                0,
                vec![BindingDescriptor {
                    name: "Camera".into(),
                    index: 0,
                    bind_type: BindType::Uniform {
                        has_dynamic_offset: false,
                        property: UniformProperty::Struct(vec![UniformProperty::Mat4]),
                    },
                    shader_stage: BindingShaderStage::VERTEX,
                }],
            )],
            vertex_buffer_descriptors: vec![VertexBufferLayout {
                name: "CanvasVertexBuffer".into(),
                stride: size_of::<Vertex>() as u64,
                step_mode: InputStepMode::Vertex,
                attributes: vec![
                    VertexAttribute {
                        name: "position".into(),
                        offset: 0,
                        format: VertexFormat::Float3,
                        shader_location: 0,
                    },
                    VertexAttribute {
                        name: "color".into(),
                        offset: VertexFormat::Float4.get_size(),
                        format: VertexFormat::Float4,
                        shader_location: 1,
                    },
                ],
            }],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: Some(IndexFormat::Uint32),
            front_face: FrontFace::Cw,
            cull_mode: CullMode::None,
            polygon_mode: PolygonMode::Fill,
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            bias: DepthBiasState {
                constant: 1,
                slope_scale: 1.,
                clamp: 1.,
            },
            clamp_depth: false,
            stencil: StencilState {
                front: StencilFaceState::IGNORE,
                back: StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
        }),
        color_target_states: vec![ColorTargetState {
            format: TextureFormat::default(),
            color_blend: BlendState {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendState {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        shader_stages: ShaderStages {
            vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER_SRC)),
            fragment: Some(shaders.add(Shader::from_glsl(
                ShaderStage::Fragment,
                FRAGMENT_SHADER_SRC,
            ))),
        },
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    }
}
