use nih_plug::{prelude::*, util::db_to_gain_fast};
use nih_plug_egui::EguiState;
use std::f32::consts;
use std::sync::{Arc, atomic::Ordering};
mod editor;

pub struct MyPlugin {
    params: Arc<PluginParams>,
    peak_meter_decay_factor: f32,
    peak_meter: Arc<AtomicF32>,

    sample_rate: f32,

    /// The current phase of the sine wave, always kept between in `[0, 1]`.
    phase: f32,

    /// The MIDI note ID of the active note, if triggered by MIDI.
    midi_note_id: u8,
    /// The frequency if the active note, if triggered by MIDI.
    midi_note_freq: f32,
    /// A simple attack and release envelope to avoid clicks. Controlled through velocity and
    /// aftertouch.
    ///
    /// Smoothing is built into the parameters, but you can also use them manually if you need to
    /// smooth soemthing that isn't a parameter.
    midi_note_gain: Smoother<f32>,
}

#[derive(Params)]
struct PluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "mute"]
    pub mute: BoolParam,

    #[id = "freq"]
    pub frequency: FloatParam,

    #[id = "usemid"]
    pub use_midi: BoolParam,
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(PluginParams::default()),
            peak_meter_decay_factor: 0.9996,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),

            sample_rate: 1.0,

            phase: 0.0,

            midi_note_id: 0,
            midi_note_freq: 1.0,
            midi_note_gain: Smoother::new(SmoothingStyle::Linear(5.0)),
        }
    }
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(400, 300),
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Linear {
                    min: -10.0,
                    max: 10.0,
                },
            )
            .with_step_size(0.1)
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB"),

            frequency: FloatParam::new(
                "Frequency",
                420.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 20_000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0))
            // We purposely don't specify a step size here, but the parameter should still be
            // displayed as if it were rounded. This formatter also includes the unit.
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            mute: BoolParam::new("Mute", false),
            use_midi: BoolParam::new("Use MIDI", false),
        }
    }
}

impl MyPlugin {
    fn calculate_sine(&mut self, frequency: f32) -> f32 {
        let phase_delta = frequency / self.sample_rate;
        let sine = (self.phase * consts::TAU).sin();

        self.phase += phase_delta;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sine
    }
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "My Plugin";
    const VENDOR: &'static str = "Yan Vdovin";
    const URL: &'static str = "https://steck.tech";
    const EMAIL: &'static str = "info@steck.tech";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            // This is also the default and can be omitted here
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = _context.next_event();
        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            // Initialize block_peak for this channel's block
            let mut block_peak = 0.0f32;

            // --- Gain Calculation (Your existing logic is fine) ---
            let gain_db = self.params.gain.smoothed.next(); // Get smoothed dB value
            let mut gain_lin = db_to_gain_fast(gain_db); // Convert dB to linear gain factor

            if self.params.mute.value() {
                gain_lin = 0.0; // Apply mute if needed
            };

            // --- Peak Meter Update Logic ---
            if self.params.editor_state.is_open() {
                // Load the currently displayed peak value
                let current_peak_meter = self.peak_meter.load(Ordering::Relaxed);

                // Decide the new peak value: attack or decay
                let new_peak_meter = if block_peak > current_peak_meter {
                    // Attack phase: Current block's peak is higher than the stored peak
                    block_peak
                } else {
                    // Decay phase: Current block's peak is lower.
                    // Apply the decay factor to the *stored* peak.
                    // Ensure `self.peak_meter_decay_factor` is set appropriately (e.g., 0.999 for slow decay)
                    current_peak_meter * self.peak_meter_decay_factor
                };

                // Prevent decaying below near-silence, avoiding tiny non-zero values indefinitely
                let new_peak_meter = if new_peak_meter < util::MINUS_INFINITY_GAIN {
                    0.0
                } else {
                    new_peak_meter
                };

                // Store the new value back into the atomic peak_meter variable
                self.peak_meter.store(new_peak_meter, Ordering::Relaxed);
            }

            let sine = if self.params.use_midi.value() {
                // Act on the next MIDI event
                while let Some(event) = next_event {
                    if event.timing() > sample_id as u32 {
                        break;
                    }

                    match event {
                        NoteEvent::NoteOn { note, velocity, .. } => {
                            self.midi_note_id = note;
                            self.midi_note_freq = util::midi_note_to_freq(note);
                            self.midi_note_gain.set_target(self.sample_rate, velocity);
                        }
                        NoteEvent::NoteOff { note, .. } if note == self.midi_note_id => {
                            self.midi_note_gain.set_target(self.sample_rate, 0.0);
                        }
                        NoteEvent::PolyPressure { note, pressure, .. }
                            if note == self.midi_note_id =>
                        {
                            self.midi_note_gain.set_target(self.sample_rate, pressure);
                        }
                        _ => (),
                    }

                    next_event = _context.next_event();
                }

                // This gain envelope prevents clicks with new notes and with released notes
                self.calculate_sine(self.midi_note_freq) * self.midi_note_gain.next()
            } else {
                let frequency = self.params.frequency.smoothed.next();
                self.calculate_sine(frequency)
            };

            // --- Apply Gain and Find Peak ---
            for sample in channel_samples {
                // Apply gain to the sample
                *sample = sine * gain_lin;

                // Check if this (gained) sample is the new peak for the block
                let sample_abs = sample.abs();
                if sample_abs > block_peak {
                    block_peak = sample_abs; // Update block_peak if this sample is larger
                }
            }
        }

        ProcessStatus::KeepAlive
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.midi_note_id = 0;
        self.midi_note_freq = 1.0;
        self.midi_note_gain.reset(0.0);
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.peak_meter.clone())
    }
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.stecktech.myplug";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("An example plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"MStecktechPlugin";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
