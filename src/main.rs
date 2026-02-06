use {
    anyhow::{Context, Result}, std::array, wgpu, winit::{
        event::{DeviceEvent, Event, MouseScrollDelta, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::KeyCode, window::{Window, WindowBuilder}
    }
};
mod renderer;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1200;


#[pollster::main]
async fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window_size = winit::dpi::PhysicalSize::new(WIDTH, HEIGHT);
    let window = WindowBuilder::new()  
        .with_inner_size(window_size)
        .with_resizable(true)
        .with_title("GPU Path Tracer".to_string())
        .build(&event_loop)?;
    window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
    let (device, queue, surface) = connect_to_gpu(&window).await?;
    let mut renderer = renderer::PathTracer::new(device, queue);

    let mut prev: [f32; 2] = [-1.0,-1.0];
    let mut mouseSens = 0.0005;
    window.set_cursor_grab(winit::window::CursorGrabMode::Locked);

    // TODO: initialize renderer

    event_loop.run(|event, control_handle| {
        
        window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
        control_handle.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_handle.exit(),
                WindowEvent::RedrawRequested => {
                    //😊
                    let frame: wgpu::SurfaceTexture = surface
                        .get_current_texture()
                        .expect("failed to get current texture");


                    let render_target = frame.texture.create_view(&wgpu::wgt::TextureViewDescriptor::default());
                    renderer.render_frame(&render_target);

                    frame.present();
                    window.request_redraw();
                },
                WindowEvent::KeyboardInput { device_id, event, is_synthetic } =>{
                    let key = event.physical_key;
                    if(key == KeyCode::ArrowRight){
                        renderer.camera.moves(0.03,0.00,0.0);
                    }
                    if(key == KeyCode::ArrowLeft){
                        renderer.camera.moves(-0.03,0.00,0.0);
                    }
                    if(key == KeyCode::ArrowUp){
                        renderer.camera.moves(-0.00,0.03,0.0);
                    }
                    if(key == KeyCode::ArrowDown){
                        renderer.camera.moves(-0.00,-0.03,0.0);
                    }

                    if(key == KeyCode::KeyQ){
                        renderer.camera.shift(0.00,0.03,0.0);
                    }
                    if(key == KeyCode::KeyA){
                        renderer.camera.shift(-0.03,0.00,0.0);
                    }
                    if(key == KeyCode::KeyE){
                        renderer.camera.shift(0.00,-0.03,0.00);
                    }
                    if(key == KeyCode::KeyD){
                        renderer.camera.shift(0.03,0.00,0.00);
                    }
                    if(key == KeyCode::KeyS){
                        renderer.camera.shift(0.00,0.00,0.03);
                    }
                    if(key == KeyCode::KeyW){
                        renderer.camera.shift(0.00,0.00,-0.03);
                    }
                    
                    
                    
                },
                WindowEvent::CursorMoved { device_id, position } =>{
                    if((prev[0] - -1.0).abs() < 0.0001){
                        prev = [position.x as f32, position.y as f32];
                    }
                    
                    renderer.camera.rotate((position.y as f32 - prev[1])*mouseSens,(position.x as f32 - prev[0])*mouseSens);
                    renderer.camera.set_w();
                    
                    prev = [position.x as f32, position.y as f32];
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseWheel { delta } => {
                    match delta{
                        MouseScrollDelta::LineDelta(x, y) => {
                            renderer.fov = (renderer.fov + y as f32 * renderer.fov).clamp(0.25, 100000.00);
                            mouseSens = (0.0005*10.0)/renderer.fov;
                            
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            // Do nothing
                        }
                    }

                },
                _ => {},
            },
            _ => (),
        }
    })?;
    Ok(())
}

async fn connect_to_gpu(
    window: &Window
) -> Result<(wgpu::Device, wgpu::Queue, wgpu::Surface)> {
    use wgpu::TextureFormat::{Bgra8Unorm, Rgba8Unorm};

    // Create an "instance" of wgpu. This is the entry-point to the API.
    let instance = wgpu::Instance::default();

    // Create a drawable "surface" that is associated with the window.
    let surface = instance.create_surface(window)?;

    // Request a GPU that is compatible with the surface. If the system has multiple GPUs then
    // pick the high performance one.
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .context("failed to find a compatible adapter")?;

    // Connect to the GPU. "device" represents the connection to the GPU and allows us to create
    // resources like buffers, textures, and pipelines. "queue" represents the command queue that
    // we use to submit commands to the GPU.
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .context("failed to connect to the GPU")?;

    // Configure the texture memory backing the surface. Our renderer will draw to a surface
    // texture every frame.
    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .into_iter()
        .find(|it| matches!(it, Rgba8Unorm | Bgra8Unorm))
        .context("could not find preferred texture format (Rgba8Unorm or Bgra8Unorm)")?;
    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 3,
    };
    surface.configure(&device, &config);

    Ok((device, queue, surface))
}
