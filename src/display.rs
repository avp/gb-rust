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
    let mut buf = [0u32; WIDTH * HEIGHT];
    for i in 0..buf.len() {
      buf[i] = ((frame[i * 4] as u32) << 16)
        | ((frame[i * 4 + 1] as u32) << 8)
        | ((frame[i * 4 + 2] as u32) << 0);
    }

    self
      .display
      .update_with_buffer(&buf, WIDTH, HEIGHT)
      .unwrap();
  }
}
