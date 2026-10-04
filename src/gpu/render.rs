use crate::gpu::config::{Config, ShaderConfig};
use eframe::egui::{PaintCallbackInfo, Vec2};
use egui_wgpu::{
    wgpu::{
        util::{BufferInitDescriptor, DeviceExt},
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
        BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages,
        ColorTargetState, CommandBuffer, CommandEncoder, Device, FragmentState, MultisampleState,
        PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
        RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexState,
    },
    CallbackResources, CallbackTrait, RenderState, ScreenDescriptor,
};

pub struct Renderer {
    pipeline: RenderPipeline,
    state: State,
}

impl Renderer {
    pub fn new(size: Vec2, cfg: &Config, shader: &str, render_state: &RenderState) -> Self {
        let device = &render_state.device;

        let config_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("shader_config"),
            contents: bytemuck::cast_slice(&[cfg.as_shader_config(size)]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("shader_config_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: config_buffer.as_entire_binding(),
            }],
        });

        let state = State {
            target_format: render_state.target_format.into(),
            bind_group_layout,
            bind_group,
            uniform_buffer: config_buffer,
        };

        let pipeline = state.generate_pipeline(device, shader);

        Self { pipeline, state }
    }

    fn prepare(&mut self, queue: &Queue, callback: &RenderCallback) {
        queue.write_buffer(
            &self.state.uniform_buffer,
            0,
            bytemuck::cast_slice(&[callback.config]),
        );
    }

    fn paint(&self, render_pass: &mut RenderPass<'static>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.state.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

pub struct State {
    target_format: ColorTargetState,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    uniform_buffer: Buffer,
}

impl State {
    pub fn generate_pipeline(&self, device: &Device, shader: &str) -> RenderPipeline {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("fv_shader"),
            source: ShaderSource::Wgsl(shader.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("fv_pipeline_layout"),
            bind_group_layouts: &[Some(&self.bind_group_layout)],
            immediate_size: 0,
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("fv_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(self.target_format.clone())],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }
}

pub struct RenderCallback {
    pub config: ShaderConfig,
}

impl CallbackTrait for RenderCallback {
    fn prepare(
        &self,
        _device: &Device,
        queue: &Queue,
        _desc: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let renderer: &mut Renderer = resources.get_mut().unwrap();
        renderer.prepare(queue, self);

        vec![]
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let renderer: &Renderer = resources.get().unwrap();
        renderer.paint(render_pass);
    }
}
