//! Render the webview

use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::ecs::prelude::*;
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::lifetimeless::Read;
use bevy::prelude::{App, Plugin};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::{
    CachedRenderPipelineId, MultisampleState, Operations, PipelineCache, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, TextureFormat,
    TextureViewDimension,
};
use bevy::render::renderer::{RenderAdapter, RenderContext};
use bevy::render::view::ViewTarget;
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderSet};
use bevy::window::{CompositeAlphaMode, PresentMode, PrimaryWindow, Window};
use bevy::{
    render::renderer::{RenderDevice, RenderInstance},
    window::RawHandleWrapper,
};
use wgpu::{TextureView, TextureViewDescriptor};

use crate::bevy_tao_loop::WebviewRawHandles;

#[derive(Resource)]
struct ExtractedWebviewHandles {
    handle: RawHandleWrapper,
    window_data: WindowData,
}
#[derive(Clone, Copy)]
struct WindowData {
    physical_width: u32,
    physical_height: u32,
    alpha_mode: CompositeAlphaMode,
    present_mode: PresentMode,
}

fn extract(
    mut cmds: Commands,
    handles: Extract<Option<Res<WebviewRawHandles>>>,
    primary_window: Extract<Query<&Window, With<PrimaryWindow>>>,
) {
    if let (Some(handles), Ok(window)) = (&*handles, primary_window.get_single()) {
        cmds.insert_resource(ExtractedWebviewHandles {
            handle: handles.0.clone(),
            window_data: WindowData {
                physical_width: window.physical_width(),
                physical_height: window.physical_height(),
                alpha_mode: window.composite_alpha_mode,
                present_mode: window.present_mode,
            },
        });
    }
}

#[derive(Resource)]
struct WebviewSurface {
    texture: wgpu::SurfaceTexture,
    format: TextureFormat,
}
impl WebviewSurface {
    fn create_view(&self) -> TextureView {
        let descr = TextureViewDescriptor {
            label: Some("webview_texture_view"),
            dimension: Some(TextureViewDimension::D2),
            format: Some(self.format.clone()),
            ..Default::default()
        };
        self.texture.texture.create_view(&descr)
    }
}

fn prepare_webview(
    mut cmds: Commands,
    render_instance: Res<RenderInstance>,
    webview: Option<Res<ExtractedWebviewHandles>>,
) {
    let Some(webview) = webview else { return; };
    // SAFETY: I don't know what I'm doing 🐕
    let surface = unsafe {
        render_instance
            .create_surface(&webview.handle.get_handle())
            .unwrap()
    };
    cmds.insert_resource(WebviewSurface {
        texture: surface.get_current_texture().unwrap(),
        format: TextureFormat::Rgba8UnormSrgb,
    });
}

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(ExtractSchedule, extract)
            .add_systems(Render, prepare_webview.in_set(RenderSet::Prepare))
            .add_render_graph_node::<ViewNodeRunner<WebviewNode>>(
                core_3d::graph::NAME,
                WebviewNode::NAME,
            )
            .add_render_graph_edges(
                core_3d::graph::NAME,
                &[
                    core_3d::graph::node::TONEMAPPING,
                    WebviewNode::NAME,
                    core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                ],
            );
    }
    fn finish(&self, app: &mut App) {
        // We need to get the render app from the main app
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Initialize the pipeline
            .init_resource::<WebviewPipeline>();
    }
}

#[derive(Default)]
struct WebviewNode;
impl WebviewNode {
    pub const NAME: &str = "webview";
}
impl ViewNode for WebviewNode {
    type ViewQuery = Read<ViewTarget>;

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        view_target: QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Some(webview_texture) = world.get_resource::<WebviewSurface>() else {
            return Ok(());
        };
        let webview_pipeline = world.resource::<WebviewPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(webview_pipeline.pipeline_id) else {
            return Ok(());
        };

        // TODO(perf): This should work as a simple additional color attachement on the main pass.
        let webview = view_target.post_process_write();

        let webview_view = webview_texture.create_view();
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("webview_pass"),
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: &webview.source,
                    resolve_target: None,
                    ops: Operations::default(),
                }),
                Some(RenderPassColorAttachment {
                    view: &webview.destination,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }),
                Some(RenderPassColorAttachment {
                    view: &webview_view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct WebviewPipeline {
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for WebviewPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("webview_pipeline".into()),
                layout: vec![],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader_vertex_state(),
                fragment: None,
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all field can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        Self { pipeline_id }
    }
}
