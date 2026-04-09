use anyhow::Result;
use hound::{SampleFormat, WavSpec, WavWriter};
use log::{debug, info};
use once_cell::sync::Lazy;
use std::io::Cursor;

static GROQ_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("Failed to build Groq HTTP client")
});

const GROQ_TRANSCRIPTION_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
const GROQ_MODEL: &str = "whisper-large-v3-turbo";

/// Build the prompt string combining custom words (for vocabulary recognition)
/// and a user-configurable base prompt (formatting / style instructions).
///
/// Whisper's `prompt` parameter is used both as a vocabulary hint and as a
/// loose instruction channel. Groq's implementation honors this field.
fn build_prompt(base_prompt: &str, custom_words: &[String]) -> String {
    if custom_words.is_empty() {
        base_prompt.to_string()
    } else {
        format!("Vocabulary: {}\n\n{}", custom_words.join(", "), base_prompt)
    }
}

/// Encode f32 audio samples as a WAV byte buffer (16-bit PCM, 16 kHz, mono).
fn encode_wav(samples: &[f32]) -> Result<Vec<u8>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut cursor, spec)?;
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            writer.write_sample((clamped * i16::MAX as f32) as i16)?;
        }
        writer.finalize()?;
    }
    Ok(cursor.into_inner())
}

#[derive(serde::Deserialize)]
struct GroqResponse {
    text: String,
}

/// Call Groq's Whisper transcription API.
///
/// - `api_key`: Groq API key (from settings)
/// - `audio`: Raw f32 audio samples at 16 kHz mono
/// - `language`: Optional ISO-639-1 language code (None = auto-detect)
/// - `custom_words`: User-configured vocabulary terms for better recognition
pub async fn transcribe_with_groq(
    api_key: &str,
    audio: &[f32],
    language: Option<&str>,
    custom_words: &[String],
    base_prompt: &str,
) -> Result<String> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "Groq API key is not configured. Please set it in Settings > Models."
        ));
    }

    let start = std::time::Instant::now();

    let wav_bytes = encode_wav(audio)?;
    debug!(
        "Encoded WAV: {} bytes from {} samples",
        wav_bytes.len(),
        audio.len()
    );

    let prompt = build_prompt(base_prompt, custom_words);

    let client = &*GROQ_CLIENT;
    let file_part = reqwest::multipart::Part::bytes(wav_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")?;

    let mut form = reqwest::multipart::Form::new()
        .text("model", GROQ_MODEL.to_string())
        .text("response_format", "json".to_string())
        .part("file", file_part);

    if let Some(lang) = language {
        form = form.text("language", lang.to_string());
    }

    if !prompt.is_empty() {
        form = form.text("prompt", prompt);
    }

    info!("Sending audio to Groq API...");
    let response = client
        .post(GROQ_TRANSCRIPTION_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Groq API error ({}): {}", status, body));
    }

    let groq_response: GroqResponse = response.json().await?;

    info!(
        "Groq transcription completed in {}ms",
        start.elapsed().as_millis()
    );

    Ok(groq_response.text)
}
