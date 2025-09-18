use crate::android::AndroidEnv;
use crate::user_input::InputHandler;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use wgpu::{
    ColorTargetState, ColorWrites, DeviceDescriptor, PipelineCompilationOptions, PresentMode,
    SurfaceConfiguration, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::android::activity::AndroidApp,
    window::WindowAttributes,
};

const SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    const pos = array(
        vec2( 0.0,  0.5),
        vec2(-0.5, -0.5),
        vec2( 0.5, -0.5)
    );
    
    return vec4f(pos[vertex_index], 0, 1);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
}

struct SurfaceState {
    surface: wgpu::Surface<'static>,
    view_format: wgpu::TextureFormat,
    alpha_mode: wgpu::CompositeAlphaMode,
}

struct App {
    /// GPU state
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    /// Redendering state
    renderer: Option<Renderer>,
    surface_state: Option<SurfaceState>,
    last_time: Instant,

    /// App state
    size: PhysicalSize<u32>,
    input_handler: &'static Mutex<InputHandler>,

    /// Android state
    android_env: Arc<Mutex<Option<AndroidEnv>>>,
}

impl App {
    async fn new(android_env: Option<AndroidEnv>) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase::default())
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_limits: wgpu::Limits {
                    max_texture_dimension_1d: 8192,
                    max_texture_dimension_2d: 8192,
                    ..wgpu::Limits::downlevel_defaults() // GL ES requires, otherwise req = 8, provide = 4
                },
                ..Default::default()
            })
            .await
            .expect("Failed to create device");

        Self {
            instance,
            device,
            adapter,
            queue,

            renderer: None,
            surface_state: None,
            last_time: Instant::now(),

            size: PhysicalSize::new(0, 0),
            input_handler: InputHandler::get(),

            android_env: Arc::new(Mutex::new(android_env)),
        }
    }

    async fn create_renderer(&mut self) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ColorTargetState {
                        format: self.surface_state.as_ref().unwrap().view_format,
                        blend: None,
                        write_mask: ColorWrites::all(),
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        self.renderer = Some(Renderer { render_pipeline });
        self.setup_input_handler();
    }

    fn setup_swapchain(&mut self) {
        let surface_state = self.surface_state.as_ref().unwrap();
        let surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_state.view_format,
            width: self.size.width,
            height: self.size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_state.alpha_mode,
            view_formats: vec![surface_state.view_format],
            desired_maximum_frame_latency: 500,
        };
        surface_state
            .surface
            .configure(&self.device, &surface_configuration);
    }

    pub fn setup_input_handler(&self) {
        let android_env_clone = self.android_env.clone();

        self.input_handler
            .lock()
            .unwrap()
            .register_handler(move |dx, dy| {
                log::info!("Get user input: dx = {}, dy = {}", dx, dy);

                // Check swipe condition
                if dx.abs() > 60.0 && dy.abs() < 50.0 {
                    if let Some(env) = &*android_env_clone.lock().unwrap() {
                        match env.vibrate(250) {
                            Ok(_) => log::info!("Vibration started successfully"),
                            Err(e) => log::error!("Failed to start vibration: {}", e)
                        }
                    }
                }
            });
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("resumed, surface view now valid");
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();
        self.size = window.inner_size();
        let surface = self.instance.create_surface(window).unwrap();
        let cap = surface.get_capabilities(&self.adapter);
        self.surface_state = Some(SurfaceState {
            surface,
            view_format: cap.formats[0],
            alpha_mode: cap.alpha_modes[0],
        });

        self.setup_swapchain();
        pollster::block_on(self.create_renderer());

        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn suspended(&mut self) {
        log::info!("suspended, surface view now invalid");

        self.renderer.take();
        self.surface_state.take();
    }

    fn resize(&mut self, window_size: PhysicalSize<u32>) {
        self.size = window_size;
        self.setup_swapchain();

        log::info!(
            "resized: h = {}, w = {}",
            window_size.height,
            window_size.width
        )
    }

    fn render(&mut self) {
        if let (Some(surface_state), Some(renderer)) = (&self.surface_state, &self.renderer) {
            let render_texture = surface_state.surface.get_current_texture().unwrap();
            let render_texture_view = render_texture
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let t = (self.last_time.elapsed().as_secs_f64() / 5.0).sin();
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &render_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: t,
                                b: 1.0 - t,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None, // 2D should be None
                    })],
                    depth_stencil_attachment: None,
                });
                rpass.set_pipeline(&renderer.render_pipeline);
                rpass.draw(0..3, 0..1);
            }

            self.queue.submit(Some(encoder.finish()));

            render_texture.present();
        }
    }
}

impl ApplicationHandler<AndroidApp> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.resumed(event_loop);
        self.render();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        log::info!("window_event: {:?}", event);

        match event {
            WindowEvent::RedrawRequested => {
                event_loop.set_control_flow(ControlFlow::Poll);
                self.render();
            }
            WindowEvent::Destroyed => {
                self.suspended();
                event_loop.set_control_flow(ControlFlow::Wait);
            }
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::Touch(touch) => {
                let mut input_handler = InputHandler::get().lock().unwrap();
                input_handler.add_event(touch.clone());
            }
            _ => {}
        }
    }
}

pub fn run(event_loop: EventLoop<AndroidApp>, android_env: Option<AndroidEnv>) {
    let mut app = pollster::block_on(App::new(android_env));

    let err = event_loop.run_app(&mut app).unwrap_err();
    log::error!("event loop error: {}", err);
}
