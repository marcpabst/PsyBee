use std::sync::{Arc, Mutex};

use rodio::{
    source::{SineWave, Source},
    Decoder, OutputStream, OutputStreamHandle, Sample, Sink,
};
use web_time::Duration;

pub struct AudioDevice {
    output_stream: Arc<OutputStream>,
    pub stream_handle: OutputStreamHandle,
}

impl Default for AudioDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDevice {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        AudioDevice {
            output_stream: Arc::new(_stream),
            stream_handle,
        }
    }
}

pub trait AudioStimulus: Send + Sync + downcast_rs::Downcast + dyn_clone::DynClone {
    /// Play the audio stimulus.
    fn play(&mut self);
    /// Stop the audio stimulus.
    fn stop(&mut self);
    /// Pause the audio stimulus.
    fn pause(&mut self);
    /// Seek to a specific time in the audio stimulus.
    fn seek(&mut self, time: f32);
    /// Reset the audio stimulus to the beginning.
    fn reset(&mut self);
    // Restart the audio stimulus from the beginning. This is identical to
    // calling `reset` followed by `play`.
    fn restart(&mut self) {
        self.reset();
        self.play();
    }
    /// Get the duration of the audio stimulus. Returns 0.0 if the duration is
    /// unknown.
    fn duration(&self) -> f32;
    /// Set the volume of the audio stimulus (0.0 to 1.0)
    fn set_volume(&mut self, volume: f32);
    /// Get the volume of the audio stimulus (0.0 to 1.0)
    fn volume(&self) -> f32;
    /// Check if the audio stimulus is playing.
    fn is_playing(&self) -> bool;
}

downcast_rs::impl_downcast!(AudioStimulus);

#[derive(Clone)]
pub struct SineWaveStimulus {
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,
}

impl SineWaveStimulus {
    pub fn new(audio_device: &AudioDevice, frequency: f32, duration: f32) -> Self {
        let stream_handle = audio_device.stream_handle.clone();
        let source = SineWave::new(frequency).take_duration(Duration::from_secs_f32(duration));
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(source);
        sink.pause();

        SineWaveStimulus {
            stream_handle,
            sink: Arc::new(Mutex::new(sink)),
        }
    }
}

impl AudioStimulus for SineWaveStimulus {
    fn play(&mut self) {
        self.sink.lock().unwrap().play();
        log::info!("Playing sine wave");
    }

    fn stop(&mut self) {
        self.sink.lock().unwrap().stop();
    }

    fn pause(&mut self) {
        self.sink.lock().unwrap().pause();
    }

    fn seek(&mut self, time: f32) {
        self.sink
            .lock()
            .unwrap()
            .try_seek(std::time::Duration::from_secs_f32(time))
            .expect("Failed to seek sine wave");
    }

    fn reset(&mut self) {
        self.sink
            .lock()
            .unwrap()
            .try_seek(std::time::Duration::from_secs_f32(0.0))
            .expect("Failed to seek sine wave");
    }

    fn duration(&self) -> f32 {
        0.0
    }

    fn set_volume(&mut self, volume: f32) {
        self.sink.lock().unwrap().set_volume(volume);
    }

    fn volume(&self) -> f32 {
        self.sink.lock().unwrap().volume()
    }

    fn is_playing(&self) -> bool {
        self.sink.lock().unwrap().empty()
    }
}

#[derive(Clone)]
pub struct FileStimulus {
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,
}

impl FileStimulus {
    pub fn new(audio_device: &AudioDevice, file_path: &str) -> Self {
        let stream_handle = audio_device.stream_handle.clone();
        let file = std::fs::File::open(file_path).unwrap();
        let source = Decoder::new(std::io::BufReader::new(file))
            .unwrap()
            .skip_duration(Duration::from_secs_f32(0.1));

        let source = NeverStop::new(Cache::new(source));

        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(source);

        sink.pause();

        FileStimulus {
            stream_handle,
            sink: Arc::new(Mutex::new(sink)),
        }
    }
}

impl AudioStimulus for FileStimulus {
    fn play(&mut self) {
        self.sink.lock().unwrap().play();
    }

    fn stop(&mut self) {
        self.sink.lock().unwrap().stop();
    }

    fn pause(&mut self) {
        self.sink.lock().unwrap().pause();
    }

    fn seek(&mut self, time: f32) {
        let _ = self
            .sink
            .lock()
            .unwrap()
            .try_seek(std::time::Duration::from_secs_f32(time));
    }

    fn reset(&mut self) {
        // if sink is stopped, it will not play again
        // so we need to append the source again
        let sink = self.sink.lock().unwrap();

        sink.try_seek(std::time::Duration::from_secs_f32(0.0))
            .expect("Failed to seek file");
    }

    fn duration(&self) -> f32 {
        0.0
    }

    fn set_volume(&mut self, volume: f32) {
        self.sink.lock().unwrap().set_volume(volume);
    }

    fn volume(&self) -> f32 {
        self.sink.lock().unwrap().volume()
    }

    fn is_playing(&self) -> bool {
        self.sink.lock().unwrap().empty()
    }
}

/// A source that appends an infinite stream of zeros to the end of the source.
#[derive(Clone, Debug)]
pub struct NeverStop<I>
where
    I: Source,
    I::Item: Sample,
{
    source: I,
}

impl<I> NeverStop<I>
where
    I: Source,
    I::Item: Sample,
{
    pub fn new(source: I) -> Self {
        NeverStop { source }
    }
}

impl<I> Iterator for NeverStop<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        // if next is None, then the source has ended and we should return 0
        Some(self.source.next().unwrap_or_else(I::Item::zero_value))
    }
}

impl<I> Source for NeverStop<I>
where
    I: Iterator + Source,
    I::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.source.try_seek(pos)
    }
}

/// A source that drains the underlying source into a buffer, then drops the
/// underlying source amd keeps the buffer in memory.
#[derive(Clone, Debug)]
pub struct Cache<I>
where
    I: Source,
    I::Item: Sample,
{
    buffer: Vec<I::Item>,
    channels: u16,
    sample_rate: u32,
    total_duration: Option<std::time::Duration>,

    current_sample: usize,
}

impl<I> Cache<I>
where
    I: Source,
    I::Item: Sample,
{
    pub fn new(source: I) -> Self {
        let channels = source.channels();
        let sample_rate = source.sample_rate();
        let total_duration = source.total_duration();

        let buffer = source.collect();

        Cache {
            buffer,
            channels,
            sample_rate,
            total_duration,
            current_sample: 0,
        }
    }
}

impl<I> Iterator for Cache<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        let sample = self.buffer.get(self.current_sample).copied();
        self.current_sample += 1;
        sample
    }
}

impl<I> Source for Cache<I>
where
    I: Source,
    I::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.total_duration
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        // work out the target sample index
        let target_sample = (pos.as_secs_f32() * self.sample_rate as f32) as usize;

        // check if the target sample is within the bounds of the buffer, if not move to
        // the end of the buffer
        if target_sample >= self.buffer.len() {
            self.current_sample = self.buffer.len();
        } else {
            self.current_sample = target_sample;
        }

        Ok(())
    }
}
