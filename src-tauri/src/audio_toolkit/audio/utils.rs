use crate::audio_toolkit::constants::WHISPER_SAMPLE_RATE;
use anyhow::Result;
use hound::{WavReader, WavSpec, WavWriter};
use log::debug;
use std::path::Path;

/// Silence appended after the captured audio so the model hears a pause.
///
/// Push-to-talk stops at key release. The 50ms release grace only rejects a
/// stray key-repeat; it is not an audio tail. VAD hangover keeps post-speech
/// audio that was recorded while the key was still down, and it does not keep
/// the microphone open after release. Recordings shorter than one second were
/// zero-padded to 1.25s, but a longer sentence was transcribed exactly as
/// captured. The tail is appended after VAD and resampling, and the ONNX
/// Parakeet batch decoder keeps those zeros. Parakeet still omits the final
/// period on a long utterance, so text cleanup adds one when the transcript
/// ends on a letter or digit.
pub const SENTENCE_END_SILENCE_MS: usize = 400;

/// Zero samples appended so an endpoint model can emit sentence-final punctuation.
pub fn sentence_end_silence() -> Vec<f32> {
    vec![0.0; sentence_end_silence_samples()]
}

pub fn sentence_end_silence_samples() -> usize {
    WHISPER_SAMPLE_RATE as usize * SENTENCE_END_SILENCE_MS / 1000
}

/// Apply the short-clip minimum pad, then the sentence-end silence tail.
/// An empty buffer stays empty so a cancelled or silent recording is not transcribed.
pub fn prepare_transcription_audio(mut samples: Vec<f32>) -> Vec<f32> {
    if samples.is_empty() {
        return samples;
    }

    let rate = WHISPER_SAMPLE_RATE as usize;
    if samples.len() < rate {
        samples.resize(rate * 5 / 4, 0.0);
    }
    samples.extend_from_slice(&sentence_end_silence());
    samples
}

/// Read a WAV file and return normalised f32 samples.
pub fn read_wav_samples<P: AsRef<Path>>(file_path: P) -> Result<Vec<f32>> {
    let reader = WavReader::open(file_path.as_ref())?;
    let samples = reader
        .into_samples::<i16>()
        .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
        .collect::<Result<Vec<f32>, _>>()?;
    Ok(samples)
}

/// Verify a WAV file by reading it back and checking the sample count.
pub fn verify_wav_file<P: AsRef<Path>>(file_path: P, expected_samples: usize) -> Result<()> {
    let reader = WavReader::open(file_path.as_ref())?;
    let actual_samples = reader.len() as usize;
    if actual_samples != expected_samples {
        anyhow::bail!(
            "WAV sample count mismatch: expected {}, got {}",
            expected_samples,
            actual_samples
        );
    }
    Ok(())
}

/// Save audio samples as a WAV file
pub fn save_wav_file<P: AsRef<Path>>(file_path: P, samples: &[f32]) -> Result<()> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(file_path.as_ref(), spec)?;

    // Convert f32 samples to i16 for WAV
    for sample in samples {
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)?;
    }

    writer.finalize()?;
    debug!("Saved WAV file: {:?}", file_path.as_ref());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_recording_gains_sentence_end_silence() {
        let spoken = WHISPER_SAMPLE_RATE as usize * 3;
        let out = prepare_transcription_audio(vec![0.25; spoken]);
        let tail = sentence_end_silence_samples();
        assert_eq!(
            tail,
            WHISPER_SAMPLE_RATE as usize * SENTENCE_END_SILENCE_MS / 1000
        );
        assert_eq!(out.len(), spoken + tail);
        assert!(out[spoken..].iter().all(|sample| *sample == 0.0));
        assert!(out[..spoken].iter().all(|sample| *sample == 0.25));
    }

    #[test]
    fn short_recording_keeps_minimum_pad_and_sentence_end_silence() {
        let out = prepare_transcription_audio(vec![0.5; 8_000]);
        let minimum = WHISPER_SAMPLE_RATE as usize * 5 / 4;
        assert_eq!(out.len(), minimum + sentence_end_silence_samples());
        assert!(out[8_000..].iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn empty_recording_is_not_padded() {
        assert!(prepare_transcription_audio(Vec::new()).is_empty());
    }
}
