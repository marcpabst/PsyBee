



pub struct PWindow {
    // the winit window
    pub window: winit::window::Window,
    // the wgpu device
    pub device: wgpu::Device,
    // the wgpu instance
    pub instance: wgpu::Instance,
    // the wgpu adapter
    pub adapter: wgpu::Adapter,
    // the wgpu queue
    pub queue: wgpu::Queue,
    // the wgpu surface
    pub surface: wgpu::Surface,
    // the wgpu surface configuration
    pub config: wgpu::SurfaceConfiguration,
    // broadcast receiver for keyboard events
    pub keyboard_receiver: async_broadcast::InactiveReceiver<winit::event::KeyboardInput>,
}
