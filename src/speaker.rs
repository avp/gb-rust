use portaudio as pa;

use std::collections::VecDeque;
use std::sync::mpsc;

pub struct Speaker {
  // Output stream.
  stream: pa::Stream<pa::NonBlocking, pa::Output<f32>>,

  sound_send: mpsc::Sender<Vec<(i8, i8)>>,
}

impl Drop for Speaker {
  fn drop(&mut self) {
    self.stream.close().expect("failed to close stream");
  }
}

fn normalize_sample(s: i8) -> f32 {
  (s as f32) * (1.0 / 127.0)
}

impl Speaker {
  pub fn new() -> Result<Speaker, pa::Error> {
    const CHANNELS: i32 = 2;
    const NUM_SECONDS: i32 = 5;
    const SAMPLE_RATE: f64 = 44_100.0;
    const FRAMES_PER_BUFFER: u32 = 64;
    const TABLE_SIZE: usize = 200;

    let pa = pa::PortAudio::new()?;

    let (sound_send, sound_recv) = mpsc::channel();

    let mut settings = pa.default_output_stream_settings(
      CHANNELS,
      SAMPLE_RATE,
      FRAMES_PER_BUFFER,
    )?;
    // We won't output out of range samples so don't bother clipping them.
    settings.flags = pa::stream_flags::CLIP_OFF;

    // This routine will be called by the PortAudio engine when audio is needed.
    // It may called at interrupt level on some machines so don't do anything
    // that could mess up the system like dynamic resource allocation or IO.
    let callback =
<<<<<<< HEAD
      move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
          buffer[idx] = sine[left_phase];
          buffer[idx + 1] = sine[right_phase];
          left_phase += 1;
          if left_phase >= TABLE_SIZE {
            left_phase -= TABLE_SIZE;
=======
      move |pa::OutputStreamCallbackArgs {
              buffer, frames, ..
            }| {
        let mut queue = VecDeque::new();
        match sound_recv.try_recv() {
          Ok(b) => {
            for (c1, c2) in b {
              queue.push_back((normalize_sample(c1), normalize_sample(c2)));
            }
>>>>>>> [APU] Skeleton for playing music.
          }
          Err(_) => {}
        }
        for i in 0..frames {
          match queue.pop_front() {
            Some((c1, c2)) => {
              buffer[i * 2] = c1;
              buffer[i * 2 + 1] = c2;
            }
            None => {
              buffer[i * 2] = 0.0;
              buffer[i * 2 + 1] = 0.0;
            }
          }
        }
        pa::Continue
      };

    let stream = pa.open_non_blocking_stream(settings, callback)?;
    let speaker = Speaker {
      stream: stream,
      sound_send: sound_send,
    };
    Ok(speaker)
  }

  pub fn play(&mut self, buf: Vec<(i8, i8)>) {
    self
      .sound_send
      .send(buf)
      .expect("Sound receiver closed");
  }

  fn start(&mut self) -> Result<(), pa::Error> {
    self.stream.start()
  }
}
