/// Options for the psydk library.
#[derive(Debug, Clone, Copy)]
pub struct GlobalOptions {
    /// The backend to use for the GPU.
    pub gpu_backend: GPUBackend,

    /// Strategy to use for blocking when rendering frames.
    pub blocking_strategy: BlockingStrategy,

    /// The maximum number of frames in flight. Should usually be set to 1.
    pub max_frames_in_flight: u32,

    /// How to check if a frame has been dropped.
    pub frame_drop_check_strategy: FrameDropCheckStrategy,

    /// How to timestamp the frames.
    pub timestamping_strategy: TimestampingStrategy,
}

#[derive(Debug, Clone, Copy)]
pub enum GPUBackend {
    /// Use the Vulkan backend (all platforms).
    Vulkan,
    /// Use the Metal backend (macOS and iOS only).
    Metal,
    /// Use the DX12 backend (Windows only).
    DirectX12,
    /// Use the OpenGL backend.
    OpenGL,
}

#[derive(Debug, Clone, Copy)]
pub enum BlockingStrategy {
    /// Will render the current frame using a command buffer and submit it to the GPU, then immediately return.
    /// Note that this may still block depending on the maximum number of frames in flight, i.e. if you submit
    /// too many frames in short succession, this will block until the GPU has caught up with the work.
    DoNotBlock,

    // Will block until the start of the vertical blanking interval at which the current frame will be presented,
    /// then return as quickly as possible. On windows, this will use the `D3DKMTGetScanLine` function to find the
    /// start of the vertical blanking interval.
    BlockUntilVBlankStart,

    /// Will block until the end of the vertical blanking interval at which the current frame will be presented,
    /// then return as quickly as possible. On windows, this will use the `D3DKMTGetScanLine` function to find the
    /// end of the vertical blanking interval, while on Vulkan it will use the `VK_GOOGLE_display_timing` extension.
    BlockUntilVBlankEnd,

    /// Will block until the end of the vertical blanking interval at which the current frame will be presented,
    /// then verify that the frame has been presented and if any frames have been missed. This will only be available
    /// on platforms that support querying the current scanline.
    BlockUntilVBlankEndVerified,
}

/// How to check if a frame has been dropped.
#[derive(Debug, Clone, Copy)]
pub enum FrameDropCheckStrategy {
    /// User statics provided by the Graphics API.
    GraphicsAPI,

    /// Use timing information to estimate if a frame has been dropped.
    Timing,
}

#[derive(Debug, Clone, Copy)]
pub enum TimestampingStrategy {
    /// Timestamp when the blocking submit call returns. The accuracy of this timestamp depends on the
    /// platform and the blocking strategy used.
    BlockingSubmit,

    /// Use the timestamps provided by the graphics API. On DirectX12, this will use the timestamps provided
    /// by `GetPresentStatistics`, on Vulkan it will use the timestamps provided by the `VK_GOOGLE_display_timing`
    /// extension.
    GraphicsAPI,

    /// Estimate the timestamp based on a regression model, using the timestamps obtained from the blocking submit call.
    /// If you observe jitter in the timestamps, you may want to use this strategy.
    BlockingSubmitEstimate,

    /// Estimate the timestamp based on a regression model, using the timestamps provided by the graphics API.
    GraphicsAPIEstimate,
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self {
            gpu_backend: GPUBackend::Vulkan,
            blocking_strategy: BlockingStrategy::BlockUntilVBlankEndVerified,
            max_frames_in_flight: 1,
            frame_drop_check_strategy: FrameDropCheckStrategy::GraphicsAPI,
            timestamping_strategy: TimestampingStrategy::BlockingSubmit,
        }
    }
}
