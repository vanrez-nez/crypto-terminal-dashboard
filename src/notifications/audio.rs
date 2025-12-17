//! Audio alerts for notifications
//!
//! Uses ALSA aplay command for lightweight audio on Raspberry Pi.
//! Supports custom WAV files from the sounds/ directory.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;

/// Separate audio channels to prevent interference
/// Alert channel for notifications
static ALERT_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
/// Tone channel for ticker price tones
static TONE_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

/// Clean up any finished audio process from the alert channel
fn cleanup_alert_process() {
    if let Ok(mut guard) = ALERT_PROCESS.lock() {
        if let Some(ref mut child) = *guard {
            // Try to reap the process if it's done
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process finished, clear it
                    *guard = None;
                }
                Ok(None) => {
                    // Still running - kill it to prevent pile-up
                    let _ = child.kill();
                    let _ = child.wait();
                    *guard = None;
                }
                Err(_) => {
                    *guard = None;
                }
            }
        }
    }
}

/// Clean up any finished audio process from the tone channel
fn cleanup_tone_process() {
    if let Ok(mut guard) = TONE_PROCESS.lock() {
        if let Some(ref mut child) = *guard {
            // Try to reap the process if it's done
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process finished, clear it
                    *guard = None;
                }
                Ok(None) => {
                    // Still running - kill it to prevent pile-up
                    let _ = child.kill();
                    let _ = child.wait();
                    *guard = None;
                }
                Err(_) => {
                    *guard = None;
                }
            }
        }
    }
}

// Simple 8-bit WAV beep (220Hz, 200ms) - generated programmatically
const BEEP_WAV: &[u8] = &[
    // RIFF header
    0x52, 0x49, 0x46, 0x46, // "RIFF"
    0x24, 0x08, 0x00, 0x00, // File size - 8
    0x57, 0x41, 0x56, 0x45, // "WAVE"
    // fmt chunk
    0x66, 0x6d, 0x74, 0x20, // "fmt "
    0x10, 0x00, 0x00, 0x00, // Chunk size (16)
    0x01, 0x00, // Audio format (1 = PCM)
    0x01, 0x00, // Num channels (1 = mono)
    0x40, 0x1f, 0x00, 0x00, // Sample rate (8000)
    0x40, 0x1f, 0x00, 0x00, // Byte rate (8000)
    0x01, 0x00, // Block align (1)
    0x08, 0x00, // Bits per sample (8)
    // data chunk
    0x64, 0x61, 0x74, 0x61, // "data"
    0x00, 0x08, 0x00, 0x00, // Data size (2048 bytes = 256ms at 8kHz)
];

const BEEP_PATH: &str = "/tmp/crypto_alert.wav";

/// Find a sound file in the sounds/ directory.
/// Search order: next to executable, then current working directory.
fn find_sound_file(filename: &str) -> Option<PathBuf> {
    // Try sounds directory next to executable
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let sound_path = exe_dir.join("sounds").join(filename);
            if sound_path.exists() {
                return Some(sound_path);
            }
        }
    }

    // Try sounds directory in current working directory
    let cwd_path = PathBuf::from("sounds").join(filename);
    if cwd_path.exists() {
        return Some(cwd_path);
    }

    None
}

/// Initialize audio by writing the fallback beep WAV file
pub fn init_audio() -> bool {
    // Generate the actual audio data (sine wave)
    let mut wav_data = BEEP_WAV.to_vec();

    // Generate 2048 samples of sine wave at ~440Hz (8kHz sample rate)
    let sample_rate = 8000.0;
    let frequency = 440.0;
    let samples = 2048;

    for i in 0..samples {
        let t = i as f32 / sample_rate;
        let sample = ((t * frequency * 2.0 * std::f32::consts::PI).sin() * 60.0 + 128.0) as u8;
        wav_data.push(sample);
    }

    // Update data chunk size in header
    let data_size = samples as u32;
    wav_data[40] = (data_size & 0xFF) as u8;
    wav_data[41] = ((data_size >> 8) & 0xFF) as u8;
    wav_data[42] = ((data_size >> 16) & 0xFF) as u8;
    wav_data[43] = ((data_size >> 24) & 0xFF) as u8;

    // Update RIFF size
    let riff_size = data_size + 36;
    wav_data[4] = (riff_size & 0xFF) as u8;
    wav_data[5] = ((riff_size >> 8) & 0xFF) as u8;
    wav_data[6] = ((riff_size >> 16) & 0xFF) as u8;
    wav_data[7] = ((riff_size >> 24) & 0xFF) as u8;

    match fs::write(BEEP_PATH, &wav_data) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Failed to write audio file: {}", e);
            false
        }
    }
}

/// Play an alert sound (non-blocking) on the alert channel
///
/// If a custom sound file is specified and found in sounds/, it will be played.
/// Otherwise, falls back to the generated beep.
/// Uses a separate audio channel from ticker tones to prevent interference.
pub fn play_alert(sound: Option<&str>) {
    // Clean up any previous alert process first
    cleanup_alert_process();

    let sound_path = match sound {
        Some(filename) => {
            // Try to find custom sound file
            find_sound_file(filename)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| BEEP_PATH.to_string())
        }
        None => BEEP_PATH.to_string(),
    };

    // Use aplay with quiet mode (suppress output)
    if let Ok(child) = Command::new("aplay").args(["-q", &sound_path]).spawn() {
        if let Ok(mut guard) = ALERT_PROCESS.lock() {
            *guard = Some(child);
        }
    }
}

/// Generate a tone WAV file at the specified frequency and duration.
/// Returns the path to the generated temporary file.
pub fn generate_tone(frequency: f32, duration_ms: u32) -> Option<String> {
    let sample_rate = 8000.0_f32;
    let num_samples = ((sample_rate * duration_ms as f32) / 1000.0) as usize;

    // Build WAV header (copy template)
    let mut wav_data = BEEP_WAV[..44].to_vec();

    // Generate sine wave samples with envelope to reduce clicks
    let fade_samples = (sample_rate * 0.005) as usize; // 5ms fade in/out
    for i in 0..num_samples {
        let t = i as f32 / sample_rate;
        // Apply envelope (fade in/out)
        let envelope = if i < fade_samples {
            i as f32 / fade_samples as f32
        } else if i > num_samples.saturating_sub(fade_samples) {
            (num_samples - i) as f32 / fade_samples as f32
        } else {
            1.0
        };
        let sample =
            ((t * frequency * 2.0 * std::f32::consts::PI).sin() * 50.0 * envelope + 128.0) as u8;
        wav_data.push(sample);
    }

    // Update data chunk size in header
    let data_size = num_samples as u32;
    wav_data[40] = (data_size & 0xFF) as u8;
    wav_data[41] = ((data_size >> 8) & 0xFF) as u8;
    wav_data[42] = ((data_size >> 16) & 0xFF) as u8;
    wav_data[43] = ((data_size >> 24) & 0xFF) as u8;

    // Update RIFF size
    let riff_size = data_size + 36;
    wav_data[4] = (riff_size & 0xFF) as u8;
    wav_data[5] = ((riff_size >> 8) & 0xFF) as u8;
    wav_data[6] = ((riff_size >> 16) & 0xFF) as u8;
    wav_data[7] = ((riff_size >> 24) & 0xFF) as u8;

    // Write to single temp file (overwrite each time)
    const TONE_PATH: &str = "/tmp/crypto_tone.wav";
    match fs::write(TONE_PATH, &wav_data) {
        Ok(_) => Some(TONE_PATH.to_string()),
        Err(_) => None,
    }
}

/// Play a ticker tone at the specified frequency (non-blocking) on the tone channel
///
/// Uses a separate audio channel from alerts to prevent interference.
pub fn play_tone(frequency: f32, duration_ms: u32) {
    // Clean up any previous tone process first
    cleanup_tone_process();

    if let Some(path) = generate_tone(frequency, duration_ms) {
        if let Ok(child) = Command::new("aplay").args(["-q", &path]).spawn() {
            if let Ok(mut guard) = TONE_PROCESS.lock() {
                *guard = Some(child);
            }
        }
    }
}
