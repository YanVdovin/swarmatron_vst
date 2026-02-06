use crate::voice::Voice;

const MAX_UNISON_VOICES: usize = 8;

struct VoiceBank {
    voices: [Voice; MAX_UNISON_VOICES],

    range: f32,
    detune: f32,
    phase_spread: f32,
}

impl VoiceBank {
    
}