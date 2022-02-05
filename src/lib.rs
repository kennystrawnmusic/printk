/*!
 A standalone, `#![no_std]`-friendly `Logger` crate.
 Based on the design of the logger built into `rust-osdev/bootloader` and can be used by default in any kernel.

 To use this crate, specify it as a dependency in your kernel's `Cargo.toml`, then initialize it.

 # Initialization
The logger needs to be initialized in order to be used. To do that, one must first create a global instance, using a crate like `conquer_once`:
```
pub static PRINTK: OnceCell<LockedPrintk> = OnceCell::uninit();
```
After this, it needs to be mapped to a framebuffer. Here's an example function demonstrating how to do that:
```
pub fn printk_init(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let kernel_logger = printk::PRINTK.get_or_init(move || printk::LockedPrintk::new(buffer, info));
    log::set_logger(kernel_logger).expect("logger already set");
    log::set_max_level(log::LevelFilter::Trace);
    log::info!("Hello, Kernel!");
}
```
 */

#![no_std]

#[allow(unused_imports)]
use {
    bootloader::{
        boot_info::{
            FrameBufferInfo,
            PixelFormat,
        }
    },
    conquer_once::{
        spin::{
            OnceCell,
        }
    },
    core::{
        fmt::{
            self,
            Write,
        },
        ptr,
    },
    noto_sans_mono_bitmap::{
        get_bitmap,
        get_bitmap_width,
        BitmapChar,
        BitmapHeight,
        FontWeight,
    },
    spinning_top::{
        Spinlock,
    }
};

/// Memory safety: need to ensure that each instance is mutexed
pub struct LockedPrintk(Spinlock<Printk>);

impl LockedPrintk {
    
    // Constructor
    #[allow(dead_code)] //TODO: use this in main.rs
    pub fn new(buf: &'static mut [u8], i: FrameBufferInfo) -> Self {
        LockedPrintk(Spinlock::new(Printk::new(buf, i)))
    }

    #[allow(dead_code)] //TODO: use this in main.rs
    pub unsafe fn force_unlock(&self) {
        self.0.force_unlock()
    }
}

impl log::Log for LockedPrintk {

    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut printk = self.0.lock();
        writeln!(printk, "{}:    {}", record.level(), record.args()).unwrap();
        printk.move_down(2);
    }

    fn flush(&self) {
        
    }
}

pub struct Printk {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
    x: usize,
    y: usize,
}

impl Printk {
    /// Creates a new empty logging interface
    #[allow(dead_code)]
    pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut printk = Self {
            buffer,
            info,
            x: 0,
            y: 0,
        };
        printk.clear();
        printk
    }

    /// Draws black-and-white pixels on the screen
    pub fn draw_grayscale(&mut self, x: usize, y: usize, intensity: u8) {

        // Pixel offset
        let poff = y * self.info.stride + x;

        let u8_intensity = {
            if intensity > 200 {
                0xf
            } else {
                0
            }
        };

        let color = match self.info.pixel_format {

            PixelFormat::RGB => { 
                [intensity, intensity, intensity/2, 0]
            },

            PixelFormat::BGR => {
                [intensity/2, intensity, intensity, 0]
            },

            PixelFormat::U8 => {
                [u8_intensity, 0, 0, 0]
            },

            //TODO: use embedded-graphics to solve this problem
            _ => panic!("Kernel panic -- not syncing: Unknown pixel format")
        };

        // Number of bytes in a pixel (4 on my machine)
        let bpp = self.info.bytes_per_pixel;

        // Byte offset: multiply bytes-per-pixel by pixel offset to obtain
        let boff = poff*bpp;

        // Copy bytes
        self.buffer[boff..(boff+bpp)].copy_from_slice(&color[..bpp]);

        // Raw pointer to buffer start ― that's why this is unsafe
        let _ = unsafe { ptr::read_volatile(&self.buffer[boff]) };

    }

    /// Renders characters from the `noto-sans-mono-bitmap` crate
    pub fn render(&mut self, rendered: BitmapChar) {
        
        // Loop through lines
        for (y, ln) in rendered.bitmap().iter().enumerate() {

            // Loop through characters on each line
            for (x, col) in ln.iter().enumerate() {

                // Use above draw_grayscale method to render each character in the bitmap
                self.draw_grayscale(self.x+x, self.y+y, *col)
            }
        }

        // Increment by width of each character
        self.x += rendered.width();
    }

    /// Moves down by `distance` number of pixels
    pub fn move_down(&mut self, distance: usize) {
        self.y += distance;
    }

    /// Moves to the beginning of a line
    pub fn home(&mut self) {
        self.x = 0;
    }

    /// Moves down one line
    pub fn next_line(&mut self) {
        self.move_down(14);
        self.home();
    }

    /// Moves to the top of the page
    pub fn page_up(&mut self) {
        self.y = 0;
    }

    /// Clears the screen
    pub fn clear(&mut self) {
        self.home();
        self.page_up();
        self.buffer.fill(0);
    }

    /// Gets the width of the framebuffer
    pub fn width(&self) -> usize {
        self.info.horizontal_resolution
    }

    /// Gets the height of the framebuffer
    pub fn height(&self) -> usize {
        self.info.vertical_resolution
    }


    pub fn putch(&mut self, c: char) {
        match c {
            '\n' => self.next_line(),
            '\r' => self.home(),
            c => {
                if self.x >= self.width() {
                    self.next_line();

                }
                const LETTER_WIDTH: usize = get_bitmap_width(FontWeight::Regular, BitmapHeight::Size14);

                if self.y >= (self.height() - LETTER_WIDTH) {
                    self.clear();
                }

                let mapped = get_bitmap(c, FontWeight::Regular, BitmapHeight::Size14).unwrap();
                self.render(mapped);
            }
        }
    }
}

unsafe impl Send for Printk {}
unsafe impl Sync for Printk {}

impl fmt::Write for Printk {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.putch(c)
        }
        Ok(())
    }
}