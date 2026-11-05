//! GIF89a compositing with offsets, transparency and disposal 2/3.
use super::*;
use std::{fs::File, io::BufReader, num::NonZeroU64, path::Path};

const MAX_TIMELINE_BYTES: usize = 128 * 1024 * 1024;
struct Frame {
    end: Duration,
    pixels: Arc<[u8]>,
}
pub(super) struct Timeline {
    width: u32,
    height: u32,
    frames: Vec<Frame>,
    duration: Duration,
    clock: Option<Instant>,
    last: Option<usize>,
}
impl Timeline {
    pub(super) fn prepare(path: &Path, control: &Control, generation: u64) -> Result<Self, String> {
        let mut options = ::gif::DecodeOptions::new();
        options.set_color_output(::gif::ColorOutput::RGBA);
        options.set_memory_limit(::gif::MemoryLimit::Bytes(
            NonZeroU64::new(MAX_FRAME_BYTES as u64).unwrap(),
        ));
        let mut reader = options
            .read_info(BufReader::new(File::open(path).map_err(|e| e.to_string())?))
            .map_err(|e| e.to_string())?;
        let (width, height) = (u32::from(reader.width()), u32::from(reader.height()));
        let size = frame_size(width, height)?;
        let background = reader
            .bg_color()
            .and_then(|index| {
                reader
                    .global_palette()
                    .and_then(|palette| palette.get(index * 3..index * 3 + 3))
            })
            .map_or([0, 0, 0, 0], |rgb| [rgb[0], rgb[1], rgb[2], 255]);
        let mut canvas = vec![0u8; size];
        let mut restore = Vec::new();
        let mut previous: Option<(::gif::DisposalMethod, u32, u32, u32, u32, [u8; 4])> = None;
        let mut frames = Vec::new();
        let mut duration = Duration::ZERO;
        while let Some(frame) = reader.read_next_frame().map_err(|e| e.to_string())? {
            if control.cancelled(generation) {
                return Err("GIF preparation cancelled".into());
            }
            if frames.len() >= 1000 || (frames.len() + 1).saturating_mul(size) > MAX_TIMELINE_BYTES
            {
                return Err("GIF timeline exceeds 1000 frames or 128 MiB".into());
            }
            if let Some((dispose, left, top, w, h, clear)) = previous {
                match dispose {
                    ::gif::DisposalMethod::Background => {
                        // Transparent GIF canvas/background for overlay playback.
                        for y in top..top + h {
                            for x in left..left + w {
                                let index = ((y * width + x) * 4) as usize;
                                canvas[index..index + 4].copy_from_slice(&clear);
                            }
                        }
                    }
                    ::gif::DisposalMethod::Previous => canvas.copy_from_slice(&restore),
                    _ => (),
                }
            }
            let (left, top, w, h) = (
                u32::from(frame.left),
                u32::from(frame.top),
                u32::from(frame.width),
                u32::from(frame.height),
            );
            if frames.is_empty() && frame.transparent.is_none() {
                for pixel in canvas.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&background);
                }
            }
            if w == 0
                || h == 0
                || left + w > width
                || top + h > height
                || frame.buffer.len() != w as usize * h as usize * 4
            {
                return Err("GIF frame exceeds logical canvas".into());
            }
            if frame.dispose == ::gif::DisposalMethod::Previous {
                restore = canvas.clone();
            }
            for y in 0..h {
                for x in 0..w {
                    let source = ((y * w + x) * 4) as usize;
                    if frame.buffer[source + 3] != 0 {
                        let destination = (((top + y) * width + left + x) * 4) as usize;
                        canvas[destination..destination + 4]
                            .copy_from_slice(&frame.buffer[source..source + 4]);
                    }
                }
            }
            // GIF zero-delay frames have no useful presentation interval; use
            // the documented POC fallback of 100ms. Other centisecond delays
            // are retained exactly.
            duration += Duration::from_millis(if frame.delay == 0 {
                100
            } else {
                u64::from(frame.delay) * 10
            });
            if duration > MAX_DURATION {
                return Err("GIF timeline exceeds ten minutes".into());
            }
            frames.push(Frame {
                end: duration,
                pixels: canvas.clone().into(),
            });
            previous = Some((
                frame.dispose,
                left,
                top,
                w,
                h,
                if frame.transparent.is_some() {
                    [0, 0, 0, 0]
                } else {
                    background
                },
            ));
        }
        if frames.is_empty() {
            return Err("GIF has no frames".into());
        }
        Ok(Self {
            width,
            height,
            frames,
            duration,
            clock: None,
            last: None,
        })
    }
    pub(super) fn duration(&self) -> Duration {
        self.duration
    }
    pub(super) fn start(&mut self) {
        self.clock = Some(Instant::now());
        self.last = None;
    }
    pub(super) fn tick(
        &mut self,
        generation: u64,
    ) -> Result<(Duration, Option<DecodedFrame>), String> {
        let position = self.clock.map_or(Duration::ZERO, |start| start.elapsed());
        let index = self
            .frames
            .partition_point(|frame| frame.end <= position)
            .min(self.frames.len() - 1);
        let frame = if self.last != Some(index) {
            self.last = Some(index);
            Some(DecodedFrame {
                generation,
                presentation_ns: if index == 0 {
                    0
                } else {
                    self.frames[index - 1].end.as_nanos() as u64
                },
                width: self.width,
                height: self.height,
                row_stride: self.width as usize * 4,
                rgba: self.frames[index].pixels.clone(),
                alpha: AlphaMode::Straight,
                color_space: ColorSpace::Srgb,
            })
        } else {
            None
        };
        Ok((position, frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn control() -> Control {
        Control {
            selection: Mutex::new(Selection {
                generation: 1,
                stopped: false,
            }),
            shutdown: AtomicBool::new(false),
        }
    }
    #[test]
    fn disposal_offsets_transparency_and_timing_are_composited() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/media/disposal.gif");
        let timeline = Timeline::prepare(&path, &control(), 1).unwrap();
        assert_eq!(timeline.duration, Duration::from_millis(700));
        let pixel = |frame: usize, x: usize, y: usize| {
            timeline.frames[frame].pixels[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4].to_vec()
        };
        assert_eq!(pixel(0, 1, 1), [0, 0, 0, 0]);
        assert_eq!(pixel(1, 1, 1), [0, 255, 0, 255]);
        assert_eq!(pixel(2, 1, 1), [0, 0, 0, 0]); // disposal 2 clears prior green
        assert_eq!(pixel(2, 0, 0), [0, 0, 255, 255]);
        assert_eq!(pixel(3, 0, 0), [255, 0, 0, 255]); // disposal 3 restores pre-blue
        assert_eq!(pixel(3, 3, 3), [0, 0, 255, 255]);
    }
}
