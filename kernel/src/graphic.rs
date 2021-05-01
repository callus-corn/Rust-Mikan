use crate::arg::{FrameBuffer, FrameBufferConfig};
use core::result::Result;

#[derive(Debug, Copy, Clone)]
pub enum Writer {
    Rgb(RGBWriter),
    Bgr(BGRWriter),
}

impl Writer {
    pub fn new_rgb(frame_buffer: FrameBuffer, frame_buffer_config: FrameBufferConfig) -> Writer {
        Writer::Rgb(RGBWriter::new(frame_buffer, frame_buffer_config))
    }

    pub fn new_bgr(frame_buffer: FrameBuffer, frame_buffer_config: FrameBufferConfig) -> Writer {
        Writer::Bgr(BGRWriter::new(frame_buffer, frame_buffer_config))
    }

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

    pub fn write(&self, x: usize, y: usize, c: PixelColor) -> Result<(), &str> {
        match self {
            Writer::Rgb(w) => w.write(x, y, c),
            Writer::Bgr(w) => w.write(x, y, c),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RGBWriter {
    frame_buffer_base: *mut u8,
    size: usize,
    pixels_per_scan_line: usize,
    vertical_resolution: usize,
    horizontal_resolution: usize,
}

impl RGBWriter {
    pub fn new(frame_buffer: FrameBuffer, frame_buffer_config: FrameBufferConfig) -> RGBWriter {
        RGBWriter {
            frame_buffer_base: frame_buffer.base,
            size: frame_buffer.size,
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

    pub fn write(&self, x: usize, y: usize, c: PixelColor) -> Result<(), &str> {
        let point = 4 * (self.pixels_per_scan_line * y + x);
        let pixel_out_of_range = point > self.size + 2
            || x >= self.horizontal_resolution
            || y >= self.vertical_resolution;
        if pixel_out_of_range {
            Err("pixel out of range")
        } else {
            unsafe {
                self.frame_buffer_base.add(point).write_volatile(c.r);
                self.frame_buffer_base.add(point + 1).write_volatile(c.g);
                self.frame_buffer_base.add(point + 2).write_volatile(c.b);
            }
            Ok(())
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BGRWriter {
    frame_buffer_base: *mut u8,
    size: usize,
    pixels_per_scan_line: usize,
    vertical_resolution: usize,
    horizontal_resolution: usize,
}

impl BGRWriter {
    pub fn new(frame_buffer: FrameBuffer, frame_buffer_config: FrameBufferConfig) -> BGRWriter {
        BGRWriter {
            frame_buffer_base: frame_buffer.base,
            size: frame_buffer.size,
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

    pub fn write(&self, x: usize, y: usize, c: PixelColor) -> Result<(), &str> {
        let point = 4 * (self.pixels_per_scan_line * y + x);
        if point > self.size + 2 {
            Err("pixel out of range")
        } else {
            unsafe {
                self.frame_buffer_base.add(point).write_volatile(c.b);
                self.frame_buffer_base.add(point + 1).write_volatile(c.g);
                self.frame_buffer_base.add(point + 2).write_volatile(c.r);
            }
            Ok(())
        }
    }
}

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
