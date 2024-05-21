use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

pub trait AudioStimulus: Send + Sync {
    /// Play the audio stimulus.
    fn play(&mut self) -> ();
    /// Stop the audio stimulus.
    fn stop(&mut self) -> ();
    /// Pause the audio stimulus.
    fn pause(&mut self) -> ();
    /// Seek to a specific time in the audio stimulus.
    fn seek(&mut self, time: f32) -> ();
    /// Get the current time of the audio stimulus.
    fn time(&self) -> f32;
    /// Get the duration of the audio stimulus. Returns 0.0 if the duration is unknown.
    fn duration(&self) -> f32;
    /// Set the volume of the audio stimulus (0.0 to 1.0)
    fn set_volume(&mut self, volume: f32) -> ();
    /// Get the volume of the audio stimulus (0.0 to 1.0)
    fn volume(&self) -> f32;
    /// Check if the audio stimulus is playing.
    fn is_playing(&self) -> bool;
}

pub struct SineWaveStimulus {
    sink: Sink,
    volume: f32,
    frequency: f32,
    position: f32,
}

impl SineWaveStimulus {
    pub fn new(frequency: f32) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(0.5);
        let source = SineWave::new(frequency);
        sink.append(source);
        sink.pause();
        SineWaveStimulus {
            sink,
            volume: 0.5,
            frequency,
            position: 0.0,
        }
    }
}

impl AudioStimulus for SineWaveStimulus {
    fn play(&mut self) -> () {
        self.sink.play();
    }

    fn stop(&mut self) -> () {
        self.sink.stop();
    }

    fn pause(&mut self) -> () {
        self.sink.pause();
    }

    fn seek(&mut self, time: f32) -> () {
        self.position = time;
    }

    fn time(&self) -> f32 {
        self.position
    }

    fn duration(&self) -> f32 {
        0.0
    }

    fn set_volume(&mut self, volume: f32) -> () {
        self.volume = volume;
        self.sink.set_volume(volume);
    }

    fn volume(&self) -> f32 {
        self.volume
    }

    fn is_playing(&self) -> bool {
        self.sink.empty()
    }
}
