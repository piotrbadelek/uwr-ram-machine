use minifb::{Window, WindowOptions};
use crate::MEMORY_SIZE;

const FRAMEBUFFER_WIDTH: usize = 320;
const FRAMEBUFFER_HEIGHT: usize = 200;
const VRAM_SIZE: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT; // 64000
const VRAM_OFFSET: usize = MEMORY_SIZE - VRAM_SIZE;

pub fn init_framebuffer() -> Window {
	let mut window = Window::new(
		"RAMulator TURBO - framebuffer",
		FRAMEBUFFER_WIDTH,
		FRAMEBUFFER_HEIGHT,
		WindowOptions {
			scale: minifb::Scale::X2, // 640x400 on screen for better visibility
			..WindowOptions::default()
		},
	).expect("Unable to create window.");
	window.set_target_fps(60);
	return window;
}

pub fn draw_framebuffer(window: &mut Window, memory: &[i32]) {
    let mut framebuffer = [0; VRAM_SIZE];
    for i in VRAM_OFFSET..MEMORY_SIZE {
        framebuffer[i - VRAM_OFFSET] = memory[i] as u32;
    }
    window.update_with_buffer(&framebuffer, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT).unwrap();
}