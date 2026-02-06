use nih_plug::{prelude::*};
use std::f32::consts;

#[derive(Clone, Copy, PartialEq, Enum)]
pub enum Waveform {
    Sine,
    Saw,
    // Triangle, Square, etc. can come later
}

pub struct Voice {
    freq: f32,
    phase: f32,
    sample_rate: f32,
}

impl Voice {
    pub fn new() -> Self {
        Self { freq: 420.0, sample_rate: 1.0, phase: 0.0, }
    }

    pub fn generate_wave(&mut self, frequency: f32, waveform: Waveform) -> f32 {
        let phase_delta = frequency / self.sample_rate;
        
        self.phase += phase_delta;
        self.phase = (self.phase + phase_delta).fract();  // .fract() = fractional part = mod 1.0

        // if self.phase >= 1.0 {
        //     self.phase -= 1.0;
        // }

        let wave: f32;

        match waveform {
            Waveform::Sine => wave = (self.phase * consts::TAU).sin(),
            Waveform::Saw => wave = self.phase - f32::floor(self.phase),
        }

        wave
    }
}
