use minifb::{Window, WindowOptions};

pub const FRAMEBUFFER_WIDTH: usize = 320;
pub const FRAMEBUFFER_HEIGHT: usize = 200;
pub const VRAM_SIZE: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT; // 64000
// NOTE: must be reachable from ELVM's 24-bit pointers. ELVM code addresses
// cell (ptr + 64) with ptr in 0..2^24, so VRAM must lie below 2^24 + 64.
// 14,000,064 sits between the jump table (13,000,000..~13,1M) and the
// downward-growing C stack (top at 16,777,279).
pub const VRAM_OFFSET: usize = 14000064;

pub fn init_framebuffer() -> Window {
	let mut window = Window::new(
		"Maszyna RAM Turbo - framebuffer",
		FRAMEBUFFER_WIDTH,
		FRAMEBUFFER_HEIGHT,
		WindowOptions {
			scale: minifb::Scale::X2, // 640x400 on screen for better visibility
			..WindowOptions::default()
		},
	).expect("Unable to create window.");
	window.set_target_fps(240); // minifb calls sleep() to make the target FPS consistent
	// obviously we don't want that, as that bottlenecks emulator performance significantly, so set it
	// to an arbitrary high numer.

	window
}

pub fn draw_framebuffer(window: &mut Window, memory: &[i32]) -> bool {
    let mut framebuffer = [0u32; VRAM_SIZE];
    for i in VRAM_OFFSET..VRAM_OFFSET + VRAM_SIZE {
        framebuffer[i - VRAM_OFFSET] = memory[i] as u32;
    }
    window.update_with_buffer(&framebuffer, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT).is_ok()
}