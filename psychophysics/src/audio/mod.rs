use std::sync::{Arc, Mutex};

use rodio::source::{SineWave, Source, Zero};
use rodio::Sample;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use web_time::Duration;

pub struct AudioDevice {
    output_stream: Arc<OutputStream>,
    pub stream_handle: OutputStreamHandle,
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

pub trait AudioStimulus:
    Send + Sync + downcast_rs::Downcast + dyn_clone::DynClone
{
    /// Play the audio stimulus.
    fn play(&mut self) -> ();
    /// Stop the audio stimulus.
    fn stop(&mut self) -> ();
    /// Pause the audio stimulus.
    fn pause(&mut self) -> ();
    /// Seek to a specific time in the audio stimulus.
    fn seek(&mut self, time: f32) -> ();
    /// Reset the audio stimulus to the beginning.
    fn reset(&mut self) -> ();
    /// Get the duration of the audio stimulus. Returns 0.0 if the duration is unknown.
    fn duration(&self) -> f32;
    /// Set the volume of the audio stimulus (0.0 to 1.0)
    fn set_volume(&mut self, volume: f32) -> ();
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

unsafe impl Send for SineWaveStimulus {}
unsafe impl Sync for SineWaveStimulus {}

impl SineWaveStimulus {
    pub fn new(audio_device: &AudioDevice, frequency: f32, duration: f32) -> Self {
        let stream_handle = audio_device.stream_handle.clone();
        let source =
            SineWave::new(frequency).take_duration(Duration::from_secs_f32(duration));
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
    fn play(&mut self) -> () {
        self.sink.lock().unwrap().play();
        println!("Playing sine wave");
    }

    fn stop(&mut self) -> () {
        self.sink.lock().unwrap().stop();
    }

    fn pause(&mut self) -> () {
        self.sink.lock().unwrap().pause();
    }

    fn seek(&mut self, time: f32) -> () {
        self.sink
            .lock()
            .unwrap()
            .try_seek(std::time::Duration::from_secs_f32(time))
            .expect("Failed to seek sine wave");
    }

    fn reset(&mut self) -> () {
        self.sink
            .lock()
            .unwrap()
            .try_seek(std::time::Duration::from_secs_f32(0.0))
            .expect("Failed to seek sine wave");
    }

    fn duration(&self) -> f32 {
        0.0
    }

    fn set_volume(&mut self, volume: f32) -> () {
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

unsafe impl Send for FileStimulus {}
unsafe impl Sync for FileStimulus {}

impl FileStimulus {
    pub fn new(audio_device: &AudioDevice, file_path: &str) -> Self {
        let stream_handle = audio_device.stream_handle.clone();
        let file = std::fs::File::open(file_path).unwrap();
        let source = Decoder::new(std::io::BufReader::new(file))
            .unwrap()
            .skip_duration(Duration::from_secs_f32(0.1));

        let source = NeverStop::new(source);

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
    fn play(&mut self) -> () {
        self.sink.lock().unwrap().play();
    }

    fn stop(&mut self) -> () {
        self.sink.lock().unwrap().stop();
    }

    fn pause(&mut self) -> () {
        self.sink.lock().unwrap().pause();
    }

    fn seek(&mut self, time: f32) -> () {
        let _ = self
            .sink
            .lock()
            .unwrap()
            .try_seek(std::time::Duration::from_secs_f32(time));
    }

    fn reset(&mut self) -> () {
        // if sink is stopped, it will not play again
        // so we need to append the source again
        let sink = self.sink.lock().unwrap();

        sink.try_seek(std::time::Duration::from_secs_f32(0.0))
            .expect("Failed to seek file");
    }

    fn duration(&self) -> f32 {
        0.0
    }

    fn set_volume(&mut self, volume: f32) -> () {
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
        Some(self.source.next().unwrap_or_else(|| I::Item::zero_value()))
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
