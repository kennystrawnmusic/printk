A standalone, `#![no_std]`-friendly `Logger` crate.
 Based on the design of the logger built into `rust-osdev/bootloader` and can be used by default in any kerne;.

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