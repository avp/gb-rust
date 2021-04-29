use crate::gpu;

const WIDTH: usize = gpu::WIDTH;
const HEIGHT: usize = gpu::HEIGHT;
const WINDOW_SCALE: u32 = 4;

pub struct Display {
  pub display: minifb::Window,
}

impl Display {
  pub fn new() -> anyhow::Result<Display> {
    let window = minifb::Window::new(
      "GB Rust",
      WIDTH,
      HEIGHT,
      minifb::WindowOptions {
        resize: true,
        scale: minifb::Scale::X2,
        scale_mode: minifb::ScaleMode::AspectRatioStretch,
        ..minifb::WindowOptions::default()
      },
    )?;

    Ok(Display { display: window })
  }

  pub fn redraw(&mut self, frame: &gpu::Frame) {
    self
      .display
      .update_with_buffer(frame, WIDTH, HEIGHT)
      .unwrap();
  }
}
