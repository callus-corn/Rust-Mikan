#[derive(Copy, Clone)]
#[repr(C)]
pub struct Argument {
    pub frame_buffer: FrameBuffer,
    pub frame_buffer_config: FrameBufferConfig,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FrameBuffer {
    pub base: *mut u8,
    pub size: usize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FrameBufferConfig {
    pub pixels_per_scan_line: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
    pub pixel_format: PixelFormat,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr,
}
