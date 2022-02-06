A standalone, `#![no_std]`-friendly `Logger` crate.
Based on the design of the logger built into the [bootloader](https://github.com/rust-osdev/bootloader) crate and meant to be used in OS kernels.

To use this crate, specify it as a dependency in your kernel's `Cargo.toml`, then initialize it.

# Initialization
The logger needs to be initialized in order to be used. To do that, one must first create a global instance, using a crate like `conquer_once`:
```
pub static PRINTK: OnceCell<LockedPrintk> = OnceCell::uninit();
```
After this, the `log` crate needs to be attached to it. This requires the following 3 steps:

1. Use the `get_or_init` method to unwrap the `LockedPrintk` object from its outer OnceCell
2. Use `log::set_logger` to tell the `log` crate what to attach to
3. Use `log::set_max_level` to tell the `log` crate how many levels of logging you want to do

Here's an example function demonstrating how this is done in pure Rust:
```
pub fn printk_init(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let kernel_logger = printk::PRINTK.get_or_init(move || printk::LockedPrintk::new(buffer, info));
    log::set_logger(kernel_logger).expect("logger already set");
    log::set_max_level(log::LevelFilter::Trace);
    log::info!("Hello, Kernel!");
}
```

# Contributing
If you have any functionality to add, feel free to [create a pull request](https://github.com/kennystrawnmusic/printk/pulls). I'll gladly test and accept it. Also, if you have any bugs to report, that is also what GitHub is for.
Also, if you're an employer in the Mission Viejo, California area, feel free to reach out to me [on Indeed](https://my.indeed.com/p/kenneths-fijbvmb). I've got push notifications set up there, so any message you send through Indeed will definitely be seen.