//! Captcha — 6-char alphanumeric challenges rendered as PNG:
//! 6 chars drawn from `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`, 10-minute TTL,
//! Redis key `cms:captcha:{id}`, verify-and-delete on match.

use rand::Rng;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

pub const CAPTCHA_SOURCE: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
pub const CAPTCHA_TTL_SECS: u64 = 600;

/// Candidate stroke hues: dark and saturated, one picked per challenge so
/// consecutive challenges do not look mechanically identical. Every entry
/// keeps all channels far below the white background, so the undulating
/// text stays high-contrast and legible.
const CAPTCHA_HUES: [[u8; 3]; 6] = [
    [0x8b, 0x1a, 0x1a], // crimson
    [0x1a, 0x1a, 0x8b], // navy
    [0x1a, 0x6b, 0x1a], // forest
    [0x5c, 0x1a, 0x5c], // plum
    [0x1a, 0x5c, 0x5c], // teal
    [0x6b, 0x3a, 0x0a], // rust
];

fn captcha_key(id: &str) -> String {
    format!("cms:captcha:{id}")
}

/// Generates a captcha: returns (id, base64 png data-url, answer).
pub async fn generate(redis: &ConnectionManager) -> Result<(String, String, String), String> {
    let mut conn = redis.clone();
    let id = uuid::Uuid::now_v7().simple().to_string();
    let (b64, chars) = render_png_base64()?;
    let _: redis::RedisResult<()> = conn
        .set_ex(captcha_key(&id), chars.clone(), CAPTCHA_TTL_SECS)
        .await;
    drop(conn);
    Ok((id, format!("data:image/png;base64,{b64}"), chars))
}

/// Verify-and-delete: a match consumes the row; anything else (missing,
/// expired, mismatched) fails.
pub async fn verify(redis: &ConnectionManager, id: &str, value: &str) -> bool {
    if id.is_empty() || value.is_empty() {
        return false;
    }
    let mut conn = redis.clone();
    let stored: Option<String> = conn.get(captcha_key(id)).await.unwrap_or(None);
    match stored {
        Some(answer) if answer == value => {
            let _: Result<i64, _> = conn.del(captcha_key(id)).await;
            true
        }
        _ => false,
    }
}

/// Destroys a challenge without verifying — the failed-gate companion of
/// [`verify`]: a mismatch (or a Redis hiccup during verify) must not leave
/// the captcha retryable within its TTL, so the gate deletes it outright.
pub async fn destroy(redis: &ConnectionManager, id: &str) {
    if id.is_empty() {
        return;
    }
    let mut conn = redis.clone();
    let _: Result<i64, _> = conn.del(captcha_key(id)).await;
}

/// Cropper-safe glyph-run envelope, in pixels across. The renderer packs
/// its six glyphs into a run and centers the crop window on that run; the
/// window is 240 px wide on a 400 px canvas, so a run wider than the
/// envelope would shear its first and last glyphs into the window's edge,
/// and a run narrower than the envelope floor would starve the centering
/// subtraction. Measured over the font's repertoire, a six-glyph run
/// lands anywhere in 120–324 px; inside the envelope the window provably
/// stays within the canvas with blank margin to spare on every side.
/// Outside draws are simply redrawn — the current font needs that for
/// 6.6% of draws, and the 32-draw budget exhausts with probability on the
/// order of 0.066³².
const RUN_MIN_PX: u32 = 96;
const RUN_MAX_PX: u32 = 232;

/// Renders the challenge as a 240x96 PNG and returns `(base64 png, answer)`.
///
/// Pipeline: six glyphs laid side by side, one horizontal sine wave so the
/// run undulates, a crop that keeps the text centered, and finally a random
/// deep hue recoloring the strokes. The answer is read back off the render,
/// so image and stored answer always agree.
///
/// The glyph pool is [`CAPTCHA_SOURCE`] intersected with the renderer
/// font's repertoire: the font cannot draw a few source glyphs (`L`), and
/// an unfiltered pick silently drops them, shortening some answers below
/// six characters.
fn render_png_base64() -> Result<(String, String), String> {
    let glyphs: Vec<char> = {
        let probe = captcha::Captcha::new();
        let supported = probe.supported_chars();
        CAPTCHA_SOURCE
            .chars()
            .filter(|c| supported.contains(c))
            .collect()
    };
    if glyphs.len() < 6 {
        return Err("captcha font repertoire covers too few glyphs".to_string());
    }
    let mut cap = (0..32)
        .find_map(|_| {
            let mut attempt = captcha::Captcha::new();
            attempt.set_chars(&glyphs);
            attempt.add_chars(6);
            let g = attempt.text_area();
            let run = g.right - g.left + 1;
            (RUN_MIN_PX..=RUN_MAX_PX).contains(&run).then_some(attempt)
        })
        .ok_or_else(|| "captcha layout retry budget exhausted".to_string())?;
    let answer = cap.chars_as_string();
    cap.apply_filter(captcha::filters::Wave::new(2.0, 12.0).horizontal());
    cap.view(240, 96);
    let hue = CAPTCHA_HUES[rand::rng().random_range(0..CAPTCHA_HUES.len())];
    cap.set_color(hue);
    let b64 = cap
        .as_base64()
        .ok_or_else(|| "captcha render".to_string())?;
    Ok((b64, answer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    /// Live-Redis pin for the challenge lifecycle. Skips unless
    /// `RUSHWIND_TEST_REDIS_URL` carries a DSN — CI runs without one and
    /// passes vacuously.
    async fn redis_of() -> Option<ConnectionManager> {
        let url = std::env::var("RUSHWIND_TEST_REDIS_URL").ok()?;
        let client = redis::Client::open(url).ok()?;
        client.get_connection_manager().await.ok()
    }

    #[test]
    fn captcha_key_is_namespaced_under_cms() {
        assert_eq!(captcha_key("abc"), "cms:captcha:abc");
    }

    #[test]
    fn render_answers_six_source_glyphs_into_a_240x96_png() {
        // The draw is random (layout retries, the hue) — a handful of
        // renders pins the invariants every draw must keep.
        for _ in 0..8 {
            let (b64, answer) = render_png_base64().unwrap();
            assert_eq!(answer.chars().count(), 6, "{answer}");
            assert!(
                answer.chars().all(|c| CAPTCHA_SOURCE.contains(c)),
                "{answer}: a glyph outside the source alphabet"
            );
            assert!(b64.starts_with("iVBORw0KGgo"), "not a PNG stream: {b64}");
            let png = base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .unwrap();
            // IHDR sits right behind the signature: the 240x96 crop the
            // front-end renders.
            assert_eq!(&png[12..16], b"IHDR");
            let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
            let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
            assert_eq!((width, height), (240, 96));
        }
    }

    #[tokio::test]
    async fn challenge_is_single_use_case_sensitive_and_explicitly_destroyable() {
        let Some(redis) = redis_of().await else {
            eprintln!("skipped: RUSHWIND_TEST_REDIS_URL not set");
            return;
        };
        let mut conn = redis.clone();
        // Empty inputs never reach the store.
        assert!(!verify(&redis, "", "").await);
        assert!(!verify(&redis, "some-id", "").await);

        let (id, image, answer) = generate(&redis).await.unwrap();
        assert!(
            image.starts_with("data:image/png;base64,iVBORw0KGgo"),
            "{image}"
        );
        // The row rides under the documented key with the 10-minute TTL.
        let stored: String = conn.get(captcha_key(&id)).await.unwrap();
        assert_eq!(stored, answer);
        let ttl: i64 = conn.ttl(captcha_key(&id)).await.unwrap();
        assert!(
            (CAPTCHA_TTL_SECS as i64 - 30..=CAPTCHA_TTL_SECS as i64).contains(&ttl),
            "ttl {ttl}"
        );
        // The comparison is exact — another case of the same answer misses.
        if answer.chars().any(|c| c.is_ascii_alphabetic()) {
            assert!(!verify(&redis, &id, &answer.to_lowercase()).await);
        }
        // The match consumes the row: a replay fails.
        assert!(verify(&redis, &id, &answer).await);
        assert!(!verify(&redis, &id, &answer).await);
        // Unknown ids fail; an empty or unknown destroy is a no-op.
        assert!(!verify(&redis, "no-such-challenge", "ABCDEF").await);
        destroy(&redis, "").await;
        destroy(&redis, "no-such-challenge").await;

        // A mismatch fails and consumes nothing ...
        let (id, _, answer) = generate(&redis).await.unwrap();
        let wrong: String = answer
            .chars()
            .map(|c| if c == 'A' { 'B' } else { 'A' })
            .collect();
        assert_ne!(wrong, answer);
        assert!(!verify(&redis, &id, &wrong).await);
        let present: i64 = conn.exists(captcha_key(&id)).await.unwrap();
        assert_eq!(present, 1);
        // ... so the failed gate's companion destroys it outright.
        destroy(&redis, &id).await;
        let present: i64 = conn.exists(captcha_key(&id)).await.unwrap();
        assert_eq!(present, 0);
        assert!(!verify(&redis, &id, &answer).await);
    }
}
