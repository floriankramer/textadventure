use std::{collections::HashMap, fmt::format};

use anyhow::{Context, Result};
use web_sys::{console, AudioContext, OscillatorNode, OscillatorType};

use crate::adventure;

pub struct SongPlayer {
    unit_length: f64,
    voices: Vec<VoicePlayer>,
    context: AudioContext,
}

impl SongPlayer {
    pub fn play(&self) {
        for voice in &self.voices {
            voice.play();
        }
    }
}

impl TryFrom<adventure::Song> for SongPlayer {
    type Error = anyhow::Error;

    fn try_from(value: adventure::Song) -> Result<Self, Self::Error> {
        let context = AudioContext::new().expect("unable to get an audio context");

        let mut voices = Vec::new();
        for voice in value.voices {
            voices.push(VoicePlayer::new(voice, value.unit_length, &context)?);
        }

        Ok(Self {
            unit_length: value.unit_length,
            voices,
            context,
        })
    }
}

struct VoicePlayer {
    unit_length: f64,
    oscillator: OscillatorNode,
    notes: Vec<Note>,
    note_pos: usize,
}

impl VoicePlayer {
    pub fn new(
        voice: adventure::SongVoice,
        unit_length: f64,
        context: &AudioContext,
    ) -> Result<Self> {
        log::info!("Creating a voice player");
        let raw_notes = voice.notes.split_whitespace();

        let pitch_indices = note_indices();
        let frequencies = frequencies();

        let rgx = regex::Regex::new("([a-z]+)(,|')?(1-9)?(.)?")
            .with_context(|| "unable to parse the regex")?;

        let mut notes = Vec::new();

        let octaves = [0.125, 0.25, 0.5, 1.0, 2.0];
        let mut last_octave = 2;
        // This might fail in some edge cases
        let mut last_pitch_idx = 6;

        let mut last_duration = unit_length / 4.0;
        for raw_note in raw_notes {
            let captures = rgx.captures(raw_note).expect("malformed note");

            let note = captures
                .get(1)
                .with_context(|| format!("missing pitch in note {}", raw_note))?;

            let frequency = if note.as_str() == "r" {
                0.0
            } else {
                let pitch_idx = *pitch_indices.get(note.as_str()).with_context(|| {
                    format!(
                        "unable to parse note pitch {} of note {}",
                        note.as_str(),
                        raw_note
                    )
                })?;

                if (pitch_idx - last_pitch_idx) > 6 {
                    if pitch_idx > 6 {
                        if last_octave + 1 < octaves.len() {
                            last_octave += 1;
                        }
                    } else if last_octave > 0 {
                        last_octave -= 1;
                    }
                }

                if let Some(octave_mod) = captures.get(2) {
                    if octave_mod.as_str() == "," {
                        if last_octave > 0 {
                            last_octave -= 1;
                        }
                    } else if last_octave < octaves.len() {
                        last_octave += 1;
                    }
                }

                last_pitch_idx = pitch_idx;

                frequencies[pitch_idx as usize] * octaves[last_octave]
            };

            if let Some(duration_capture) = captures.get(3) {
                last_duration = unit_length
                    / duration_capture.as_str().parse::<u64>().with_context(|| {
                        format!(
                            "durations need to be unsigned integers, got {} in note {raw_note}",
                            duration_capture.as_str()
                        )
                    })? as f64;

                // If there is a dot after the note, make it half as long again
                if captures.get(4).is_some() {
                    last_duration *= 1.5;
                }
            }

            notes.push(Note {
                frequency,
                duration: last_duration,
            });

            log::info!(
                "Adding note with frequency {}, idx {}",
                frequency,
                last_pitch_idx
            );
        }

        log::info!("Create an oscillator");
        let oscillator = context
            .create_oscillator()
            .expect("unable to create an oscillator");
        oscillator
            .connect_with_audio_node(&context.destination())
            .expect("Unable to set the oscialltor output");

        Ok(Self {
            unit_length,
            oscillator,
            notes,
            note_pos: 0,
        })
    }

    fn play(&self) {
        self.oscillator.frequency().set_value(0.0);
        self.oscillator.set_type(OscillatorType::Square);

        let mut offset = 0.0;
        for note in &self.notes {
            self.oscillator
                .frequency()
                .set_value_at_time(note.frequency, offset)
                .expect("uanble to schedule the note");
            self.oscillator
                .frequency()
                .set_value_at_time(0.0, offset + note.duration - self.unit_length / 32.0)
                .expect("uanble to schedule the note");
            offset += note.duration;
        }
        self.oscillator
            .frequency()
            .set_value_at_time(0.0, offset)
            .expect("uanble to schedule the note");

        self.oscillator
            .start()
            .expect("Unable to start the oscillator");
    }
}

struct Note {
    frequency: f32,
    duration: f64,
}

fn note_indices() -> HashMap<String, i64> {
    HashMap::from([
        ("c".to_string(), 0),
        ("cis".to_string(), 1),
        ("des".to_string(), 1),
        ("d".to_string(), 2),
        ("dis".to_string(), 3),
        ("es".to_string(), 3),
        ("e".to_string(), 4),
        ("f".to_string(), 5),
        ("fis".to_string(), 6),
        ("ges".to_string(), 6),
        ("g".to_string(), 7),
        ("gis".to_string(), 8),
        ("as".to_string(), 8),
        ("a".to_string(), 9),
        ("ais".to_string(), 10),
        ("bes".to_string(), 10),
        ("b".to_string(), 11),
    ])
}

fn frequencies() -> Vec<f32> {
    vec![
        261.0, 277.0, 293.0, 311.0, 329.0, 349.0, 369.0, 392.0, 415.0, 440.0, 466.0, 493.0,
    ]
}
