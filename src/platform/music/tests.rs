use super::*;

fn line_beats(line: Line) -> f32 {
    line.iter().map(|(_, beats)| beats).sum()
}

fn chord_beats(chords: &[Chord]) -> f32 {
    chords.iter().map(|chord| chord.beats).sum()
}

#[test]
fn battle_sections_fill_exactly_eight_bars() {
    assert_eq!(chord_beats(BATTLE_A_CHORDS), SECTION_BEATS);
    assert_eq!(chord_beats(BATTLE_B_CHORDS), SECTION_BEATS);
    assert_eq!(line_beats(BATTLE_MELODY_A), SECTION_BEATS);
    assert_eq!(line_beats(BATTLE_MELODY_B), SECTION_BEATS);
    assert_eq!(SECTION_BEATS * 4.0, MusicTrack::Battle.beats());
}

#[test]
fn menu_melody_matches_its_chords_and_track_length() {
    assert_eq!(chord_beats(MENU_CHORDS), MusicTrack::Menu.beats());
    assert_eq!(line_beats(MENU_MELODY), MusicTrack::Menu.beats());
}

#[test]
fn menu_and_battle_loop_for_a_minute_and_intro_covers_the_crawl() {
    assert_eq!(MusicTrack::Menu.duration(), 64.0);
    assert_eq!(MusicTrack::Battle.duration(), 64.0);
    assert!((MusicTrack::Intro.duration() - 56.0).abs() < 0.01);
}

#[test]
fn intro_sections_fill_the_track_exactly() {
    assert_eq!(chord_beats(INTRO_FANFARE_CHORDS), INTRO_FANFARE_BEATS);
    assert_eq!(line_beats(INTRO_FANFARE_LINE), INTRO_FANFARE_BEATS);
    assert_eq!(chord_beats(INTRO_CRAWL_CHORDS), INTRO_CRAWL_BEATS);
    assert_eq!(line_beats(INTRO_CRAWL_LINE), INTRO_CRAWL_BEATS);
    assert_eq!(chord_beats(INTRO_FINALE_CHORDS), INTRO_FINALE_BEATS);
    assert_eq!(line_beats(INTRO_FINALE_LINE), INTRO_FINALE_BEATS);
    let total = INTRO_FANFARE_START + INTRO_FANFARE_BEATS + INTRO_CRAWL_BEATS + INTRO_FINALE_BEATS;
    assert_eq!(total, MusicTrack::Intro.beats());
}

#[test]
fn intro_fanfare_lands_when_the_title_appears() {
    let start = INTRO_FANFARE_START * MusicTrack::Intro.beat();
    assert!((start - 7.5).abs() < 0.01);
}

#[test]
fn intro_is_quiet_until_the_fanfare_then_loud() {
    let samples = synthesize_track(MusicTrack::Intro);
    let rate = SAMPLE_RATE as usize;
    let loudness = |from: usize, to: usize| {
        let part = &samples[from * rate..to * rate];
        (part.iter().map(|s| s * s).sum::<f32>() / part.len() as f32).sqrt()
    };
    let prelude = loudness(0, 7);
    let fanfare = loudness(9, 28);
    let crawl = loudness(32, 50);
    assert!(fanfare > prelude * 2.5, "{prelude} {fanfare}");
    assert!(fanfare > crawl, "{fanfare} {crawl}");
}

#[test]
fn every_track_is_audible_finite_and_within_range() {
    for track in MusicTrack::ALL {
        let samples = synthesize_track(track);
        assert_eq!(samples.len(), count(track.duration()), "{:?}", track);
        assert!(
            samples.iter().all(|sample| sample.is_finite()),
            "{:?}",
            track
        );
        let peak = samples
            .iter()
            .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
        assert!(peak > 0.5 && peak <= PEAK + 1e-4, "{:?} {}", track, peak);
        let energy =
            samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32;
        assert!(energy.sqrt() > 0.02, "{:?}", track);
    }
}

#[test]
fn battle_builds_up_from_the_intro_to_the_final_section() {
    let samples = synthesize_track(MusicTrack::Battle);
    let section = samples.len() / 4;
    let loudness =
        |part: &[f32]| (part.iter().map(|s| s * s).sum::<f32>() / part.len() as f32).sqrt();
    let intro = loudness(&samples[..section]);
    let finale = loudness(&samples[section * 3..]);
    assert!(finale > intro * 1.5, "{intro} {finale}");
}
