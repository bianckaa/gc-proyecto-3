use raylib::prelude::{RaylibAudio, Sound};
use std::f32::consts::TAU;

const SAMPLE_RATE: u32 = 22_050;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEffect {
    Shot,
    Laser,
    Launch,
    Impact,
    EnemyDestroyed,
    BombExplosion,
    PlanetDestroyed,
    LevelUp,
    RouletteTick,
    Damage,
    Alarm,
    Collapse,
}

impl SoundEffect {
    pub const ALL: [SoundEffect; 12] = [
        SoundEffect::Shot,
        SoundEffect::Laser,
        SoundEffect::Launch,
        SoundEffect::Impact,
        SoundEffect::EnemyDestroyed,
        SoundEffect::BombExplosion,
        SoundEffect::PlanetDestroyed,
        SoundEffect::LevelUp,
        SoundEffect::RouletteTick,
        SoundEffect::Damage,
        SoundEffect::Alarm,
        SoundEffect::Collapse,
    ];

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|effect| *effect == self)
            .unwrap_or(0)
    }

    fn min_interval(self) -> f32 {
        match self {
            SoundEffect::Shot => 0.05,
            SoundEffect::Impact => 0.04,
            SoundEffect::EnemyDestroyed => 0.06,
            _ => 0.0,
        }
    }
}

struct Noise {
    state: u32,
}

impl Noise {
    fn next(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.state >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
}

fn render(duration: f32, mut sample: impl FnMut(f32, f32) -> f32) -> Vec<f32> {
    let count = (duration * SAMPLE_RATE as f32) as usize;
    (0..count)
        .map(|index| {
            let time = index as f32 / SAMPLE_RATE as f32;
            sample(time, time / duration)
        })
        .collect()
}

fn sweep(start: f32, end: f32, shape: fn(f32) -> f32) -> impl FnMut(f32, f32) -> f32 {
    let mut phase = 0.0_f32;
    let step = 1.0 / SAMPLE_RATE as f32;
    move |_, progress| {
        let frequency = start + (end - start) * progress;
        phase = (phase + frequency * step).fract();
        shape(phase)
    }
}

fn sine(phase: f32) -> f32 {
    (phase * TAU).sin()
}

fn square(phase: f32) -> f32 {
    if phase < 0.5 {
        1.0
    } else {
        -1.0
    }
}

fn saw(phase: f32) -> f32 {
    phase * 2.0 - 1.0
}

fn low_noise(seed: u32, smoothing: f32) -> impl FnMut() -> f32 {
    let mut noise = Noise { state: seed };
    let mut value = 0.0;
    move || {
        value += (noise.next() - value) * smoothing;
        value
    }
}

pub fn synthesize(effect: SoundEffect) -> Vec<f32> {
    match effect {
        SoundEffect::Shot => {
            let mut tone = sweep(900.0, 300.0, square);
            render(0.08, |time, progress| tone(time, progress) * (1.0 - progress) * 0.18)
        }
        SoundEffect::Laser => {
            let mut tone = sweep(1400.0, 180.0, saw);
            render(0.4, |time, progress| {
                let vibrato = 1.0 + 0.3 * (time * 60.0 * TAU).sin();
                tone(time, progress) * (1.0 - progress).powi(2) * 0.28 * vibrato
            })
        }
        SoundEffect::Launch => {
            let mut tone = sweep(320.0, 110.0, sine);
            render(0.16, |time, progress| tone(time, progress) * (1.0 - progress) * 0.35)
        }
        SoundEffect::Impact => {
            let mut noise = Noise { state: 7 };
            let mut previous = 0.0;
            render(0.05, |_, progress| {
                let value = noise.next();
                let crisp = value - previous;
                previous = value;
                crisp * (1.0 - progress) * 0.22
            })
        }
        SoundEffect::EnemyDestroyed => {
            let mut rumble = low_noise(11, 0.35);
            let mut tone = sweep(160.0, 60.0, sine);
            render(0.35, |time, progress| {
                (rumble() * 0.8 + tone(time, progress) * 0.4) * (1.0 - progress).powi(2) * 0.55
            })
        }
        SoundEffect::BombExplosion => {
            let mut rumble = low_noise(23, 0.2);
            render(0.6, |_, progress| rumble() * (1.0 - progress).powi(2) * 0.8)
        }
        SoundEffect::PlanetDestroyed => {
            let mut rumble = low_noise(31, 0.08);
            let mut tone = sweep(90.0, 35.0, sine);
            render(1.4, |time, progress| {
                (rumble() * 1.4 + tone(time, progress) * 0.5) * (1.0 - progress).powf(1.5) * 0.75
            })
        }
        SoundEffect::LevelUp => {
            let notes = [523.25, 659.25, 783.99, 1046.5];
            let mut phase = 0.0_f32;
            render(0.48, |time, _| {
                let index = ((time / 0.12) as usize).min(notes.len() - 1);
                let local = (time - index as f32 * 0.12) / 0.12;
                phase = (phase + notes[index] / SAMPLE_RATE as f32).fract();
                square(phase) * (1.0 - local * 0.7) * 0.16
            })
        }
        SoundEffect::RouletteTick => {
            let mut tone = sweep(1900.0, 1500.0, sine);
            render(0.03, |time, progress| tone(time, progress) * (1.0 - progress) * 0.3)
        }
        SoundEffect::Damage => {
            let mut noise = Noise { state: 41 };
            let mut tone = sweep(160.0, 90.0, square);
            render(0.22, |time, progress| {
                (tone(time, progress) * 0.6 + noise.next() * 0.4) * (1.0 - progress) * 0.3
            })
        }
        SoundEffect::Alarm => {
            let mut phase = 0.0_f32;
            render(1.2, |time, _| {
                let high = ((time / 0.15) as usize) % 2 == 0;
                let frequency = if high { 880.0 } else { 660.0 };
                phase = (phase + frequency / SAMPLE_RATE as f32).fract();
                square(phase) * 0.14
            })
        }
        SoundEffect::Collapse => {
            let mut rumble = low_noise(57, 0.03);
            let mut tone = sweep(55.0, 22.0, sine);
            render(3.0, |time, progress| {
                let swell = (progress * 3.0).min(1.0) * (1.0 - progress).sqrt();
                (rumble() * 1.8 + tone(time, progress) * 0.6) * swell * 0.8
            })
        }
    }
}

pub fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data_size as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

pub struct Audio {
    sounds: Vec<Sound<'static>>,
    cooldowns: [f32; SoundEffect::ALL.len()],
}

impl Audio {
    pub fn open() -> Option<Self> {
        let device = RaylibAudio::init_audio_device().ok()?;
        let device: &'static RaylibAudio = Box::leak(Box::new(device));
        let mut sounds = Vec::with_capacity(SoundEffect::ALL.len());
        for effect in SoundEffect::ALL {
            let bytes = wav_bytes(&synthesize(effect));
            let wave = device.new_wave_from_memory(".wav", &bytes).ok()?;
            sounds.push(device.new_sound_from_wave(&wave).ok()?);
        }
        Some(Self {
            sounds,
            cooldowns: [0.0; SoundEffect::ALL.len()],
        })
    }

    pub fn update(&mut self, delta_time: f32) {
        for cooldown in &mut self.cooldowns {
            *cooldown = (*cooldown - delta_time).max(0.0);
        }
    }

    pub fn play(&mut self, effect: SoundEffect) {
        let index = effect.index();
        if self.cooldowns[index] > 0.0 {
            return;
        }
        self.cooldowns[index] = effect.min_interval();
        self.sounds[index].play();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_effect_is_audible_and_within_range() {
        for effect in SoundEffect::ALL {
            let samples = synthesize(effect);
            assert!(!samples.is_empty(), "{:?}", effect);
            assert!(samples.iter().any(|sample| sample.abs() > 0.01), "{:?}", effect);
            assert!(samples.iter().all(|sample| sample.is_finite()), "{:?}", effect);
        }
    }

    #[test]
    fn wav_header_describes_the_samples() {
        let bytes = wav_bytes(&[0.0, 0.5, -0.5]);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(bytes.len(), 44 + 6);
        assert_eq!(u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]), 6);
    }
}
