use super::audio::SAMPLE_RATE;
use std::f32::consts::TAU;

const PEAK: f32 = 0.85;
const REST: i32 = i32::MIN;
const SECTION_BEATS: f32 = 32.0;
const SECTION_GAINS: [f32; 4] = [0.45, 0.7, 0.9, 1.0];
const INTRO_FANFARE_START: f32 = 11.25;
const INTRO_FANFARE_BEATS: f32 = 32.0;
const INTRO_CRAWL_BEATS: f32 = 32.0;
const INTRO_FINALE_BEATS: f32 = 8.75;
const INTRO_FILE: &str = "assets/music/intro.ogg";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicTrack {
    Menu,
    Battle,
    Intro,
}

impl MusicTrack {
    pub const ALL: [MusicTrack; 3] = [MusicTrack::Menu, MusicTrack::Battle, MusicTrack::Intro];

    pub fn index(self) -> usize {
        match self {
            MusicTrack::Menu => 0,
            MusicTrack::Battle => 1,
            MusicTrack::Intro => 2,
        }
    }

    fn beat(self) -> f32 {
        match self {
            MusicTrack::Menu => 1.0,
            MusicTrack::Battle => 0.5,
            MusicTrack::Intro => 2.0 / 3.0,
        }
    }

    fn beats(self) -> f32 {
        match self {
            MusicTrack::Menu => 64.0,
            MusicTrack::Battle => 128.0,
            MusicTrack::Intro => 84.0,
        }
    }

    pub fn external_file(self) -> Option<&'static str> {
        match self {
            MusicTrack::Intro => Some(INTRO_FILE),
            MusicTrack::Menu | MusicTrack::Battle => None,
        }
    }

    pub fn duration(self) -> f32 {
        self.beats() * self.beat()
    }
}

pub fn synthesize_track(track: MusicTrack) -> Vec<f32> {
    let mut mix = match track {
        MusicTrack::Menu => menu_mix(),
        MusicTrack::Battle => battle_mix(),
        MusicTrack::Intro => intro_mix(),
    };
    reverb(&mut mix.samples);
    let peak = mix
        .samples
        .iter()
        .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
    if peak > f32::EPSILON {
        let scale = PEAK / peak;
        for sample in &mut mix.samples {
            *sample *= scale;
        }
    }
    mix.samples
}

mod battle;
mod intro;
mod menu;
mod score;
mod synth;
#[cfg(test)]
mod tests;

use battle::battle_mix;
use intro::intro_mix;
use menu::menu_mix;
use score::*;
use synth::*;
