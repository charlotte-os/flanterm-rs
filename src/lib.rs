#![no_std]

pub mod bindings;

use bindings::*;
use core::fmt::{self, Write};
use core::mem::MaybeUninit;
use core::ptr;
use spin::mutex::Mutex;

/// A safe wrapper around the flanterm context
pub struct FlantermContext {
    ctx: *mut flanterm_context,
}

impl FlantermContext {
    /// Create a new framebuffer-based flanterm context
    pub fn new_fb(
        framebuffer: *mut u32,
        width: usize,
        height: usize,
        pitch: usize,
        red_mask_size: u8,
        red_mask_shift: u8,
        green_mask_size: u8,
        green_mask_shift: u8,
        blue_mask_size: u8,
        blue_mask_shift: u8,
    ) -> Option<Self> {
        let ctx = unsafe {
            flanterm_fb_init(
                None, // malloc
                None, // free
                framebuffer,
                width,
                height,
                pitch,
                red_mask_size,
                red_mask_shift,
                green_mask_size,
                green_mask_shift,
                blue_mask_size,
                blue_mask_shift,
                ptr::null_mut(), // canvas
                ptr::null_mut(), // ansi_colours
                ptr::null_mut(), // ansi_bright_colours
                ptr::null_mut(), // default_bg
                ptr::null_mut(), // default_fg
                ptr::null_mut(), // default_bg_bright
                ptr::null_mut(), // default_fg_bright
                ptr::null_mut(), // font
                0,               // font_width
                0,               // font_height
                1,               // font_spacing
                1,               // font_scale_x
                1,               // font_scale_y
                0,               // margin
            )
        };

        if ctx.is_null() {
            None
        } else {
            Some(Self { ctx })
        }
    }

    /// Get terminal dimensions (columns, rows)
    pub fn get_dimensions(&self) -> (usize, usize) {
        let mut cols = 0;
        let mut rows = 0;
        unsafe {
            flanterm_get_dimensions(self.ctx, &mut cols, &mut rows);
        }
        (cols, rows)
    }

    /// Set autoflush behavior
    pub fn set_autoflush(&mut self, enabled: bool) {
        unsafe {
            flanterm_set_autoflush(self.ctx, enabled);
        }
    }

    /// Flush the terminal output
    pub fn flush(&mut self) {
        unsafe {
            flanterm_flush(self.ctx);
        }
    }

    /// Force a full refresh
    pub fn full_refresh(&mut self) {
        unsafe {
            flanterm_full_refresh(self.ctx);
        }
    }

    /// Write raw bytes to the terminal
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        unsafe {
            flanterm_write(self.ctx, bytes.as_ptr() as *const i8, bytes.len());
        }
    }

    /// Clear the terminal
    pub fn clear(&mut self) {
        self.write_str("\x1b[2J\x1b[H").unwrap();
    }

    /// Move cursor to position (x, y)
    pub fn move_cursor(&mut self, x: usize, y: usize) {
        let _ = write!(self, "\x1b[{};{}H", y + 1, x + 1);
    }

    /// Set text color using ANSI codes
    pub fn set_color(&mut self, fg: u8, bg: Option<u8>) {
        if let Some(bg) = bg {
            let _ = write!(self, "\x1b[38;5;{}m\x1b[48;5;{}m", fg, bg);
        } else {
            let _ = write!(self, "\x1b[38;5;{}m", fg);
        }
    }

    /// Reset text formatting
    pub fn reset_format(&mut self) {
        self.write_str("\x1b[0m").unwrap();
    }

    /// Get a reference to the raw flanterm context pointer (unsafe)
    pub unsafe fn as_raw(&self) -> *mut flanterm_context {
        self.ctx
    }
}

impl Write for FlantermContext {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}

impl Drop for FlantermContext {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe {
                flanterm_deinit(self.ctx);
            }
        }
    }
}

// Only Send, not Sync - the Mutex provides the Sync behavior
unsafe impl Send for FlantermContext {}

/// Global flanterm state protected by a mutex
struct GlobalFlantermState {
    ctx: MaybeUninit<FlantermContext>,
    initialized: bool,
}

impl GlobalFlantermState {
    const fn new() -> Self {
        Self {
            ctx: MaybeUninit::uninit(),
            initialized: false,
        }
    }
}

/// Global flanterm instance protected by a spin mutex
static GLOBAL_FLANTERM: Mutex<GlobalFlantermState> = Mutex::new(GlobalFlantermState::new());

/// Initialize the global flanterm instance
pub fn init_global_flanterm(ctx: FlantermContext) {
    let mut state = GLOBAL_FLANTERM.lock();
    state.ctx.write(ctx);
    state.initialized = true;
}

/// Get a mutable reference to the global flanterm instance
pub fn with_global_flanterm<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut FlantermContext) -> R,
{
    let mut state = GLOBAL_FLANTERM.lock();
    if state.initialized {
        let ctx = unsafe { state.ctx.assume_init_mut() };
        Some(f(ctx))
    } else {
        None
    }
}

/// Print to the global flanterm instance
pub fn _print(args: fmt::Arguments) {
    with_global_flanterm(|ctx| {
        let _ = ctx.write_fmt(args);
    });
}

/// Print implementation for flanterm
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*))
    };
}

/// Print with newline implementation for flanterm
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}
