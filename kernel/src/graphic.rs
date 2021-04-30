use crate::arg::{FrameBuffer, FrameBufferConfig};

pub enum Writer {
    Rgb(RGBWriter),
    Bgr(BGRWriter),
}

impl Writer {
    pub fn vertical_resolution(&self) -> usize {
        match self {
            Writer::Rgb(w) => w.vertical_resolution(),
            Writer::Bgr(w) => w.vertical_resolution(),
        }
    }

    pub fn horizontal_resolution(&self) -> usize {
        match self {
            Writer::Rgb(w) => w.horizontal_resolution(),
            Writer::Bgr(w) => w.horizontal_resolution(),
        }
    }

    pub fn write(&self, x: usize, y: usize, c: PixelColor) {
        match self {
            //安全性はframe_buffer_base依存
            Writer::Rgb(w) => unsafe { w.write(x, y, c) },
            Writer::Bgr(w) => unsafe { w.write(x, y, c) },
        }
    }
}

pub struct RGBWriter {
    frame_buffer_base: *mut u8,
    //size: usize,
    pixels_per_scan_line: usize,
    vertical_resolution: usize,
    horizontal_resolution: usize,
}

impl RGBWriter {
    pub fn new(frame_buffer: FrameBuffer, frame_buffer_config: FrameBufferConfig) -> RGBWriter {
        RGBWriter {
            frame_buffer_base: frame_buffer.base,
            //size: frame_buffer.size,
            pixels_per_scan_line: frame_buffer_config.pixels_per_scan_line,
            vertical_resolution: frame_buffer_config.vertical_resolution,
            horizontal_resolution: frame_buffer_config.horizontal_resolution,
        }
    }

    pub fn vertical_resolution(&self) -> usize {
        self.vertical_resolution
    }

    pub fn horizontal_resolution(&self) -> usize {
        self.horizontal_resolution
    }

    pub unsafe fn write(&self, x: usize, y: usize, c: PixelColor) {
        let point = 4 * (self.pixels_per_scan_line * y + x);
        self.frame_buffer_base.add(point).write_volatile(c.r);
        self.frame_buffer_base.add(point + 1).write_volatile(c.g);
        self.frame_buffer_base.add(point + 2).write_volatile(c.b);
    }
}

pub struct BGRWriter {
    frame_buffer_base: *mut u8,
    //size: usize,
    pixels_per_scan_line: usize,
    vertical_resolution: usize,
    horizontal_resolution: usize,
}

impl BGRWriter {
    pub fn new(frame_buffer: FrameBuffer, frame_buffer_config: FrameBufferConfig) -> BGRWriter {
        BGRWriter {
            frame_buffer_base: frame_buffer.base,
            //size: frame_buffer.size,
            pixels_per_scan_line: frame_buffer_config.pixels_per_scan_line,
            vertical_resolution: frame_buffer_config.vertical_resolution,
            horizontal_resolution: frame_buffer_config.horizontal_resolution,
        }
    }

    pub fn vertical_resolution(&self) -> usize {
        self.vertical_resolution
    }

    pub fn horizontal_resolution(&self) -> usize {
        self.horizontal_resolution
    }

    pub unsafe fn write(&self, x: usize, y: usize, c: PixelColor) {
        let point = 4 * (self.pixels_per_scan_line * y + x);
        self.frame_buffer_base.add(point).write_volatile(c.b);
        self.frame_buffer_base.add(point + 1).write_volatile(c.g);
        self.frame_buffer_base.add(point + 2).write_volatile(c.r);
    }
}

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
