//! Whether the terminal behind the view is light or dark. `COLORFGBG` is
//! the cheap answer; failing that the terminal is asked with `OSC 11` and
//! given 100 ms. No answer means dark, which is what every terminal was
//! assumed to be before the palette had variants.

use super::colour::Background;
use std::time::Duration;

/// How long a terminal gets to answer `OSC 11`.
pub const PROBE_TIMEOUT: Duration = Duration::from_millis(100);

const QUERY: &[u8] = b"\x1b]11;?\x1b\\";

/// `COLORFGBG` is `fg;bg` or `fg;default;bg`; the last field is the
/// background index, and 7 or 15 are the light ones.
pub fn background_from_colorfgbg(value: &str) -> Option<Background> {
    let bg: u8 = value.rsplit(';').next()?.trim().parse().ok()?;
    Some(match bg {
        7 | 15 => Background::Light,
        _ => Background::Dark,
    })
}

/// The `rgb:rrrr/gggg/bbbb` reply to `OSC 11`, classified by relative
/// luminance: above one half is light.
pub fn background_from_reply(reply: &str) -> Option<Background> {
    let rgb = reply.split("rgb:").nth(1)?;
    let rgb = rgb.trim_end_matches(['\x07', '\x1b', '\\']);
    let mut parts = rgb.split('/');
    let mut channel = || -> Option<f64> {
        let hex = parts.next()?.trim();
        let max = 16f64.powi(hex.len() as i32) - 1.0;
        let value = u32::from_str_radix(hex, 16).ok()?;
        Some(f64::from(value) / max)
    };
    let (r, g, b) = (channel()?, channel()?, channel()?);
    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    Some(if luminance > 0.5 {
        Background::Light
    } else {
        Background::Dark
    })
}

/// The complete `OSC 11` reply inside `bytes`, if it has arrived. The
/// terminator is BEL or ST.
pub fn take_reply(bytes: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(bytes).ok()?;
    let start = text.find("\x1b]11;")?;
    let body = &text[start + 5..];
    let end = body.find('\x07').or_else(|| body.find("\x1b\\"))?;
    Some(&body[..end])
}

/// The background from the environment, then the terminal, then the
/// default. `probe` says whether the terminal may be asked, which needs
/// raw mode and a tty on both ends.
pub fn detect_background(probe: bool) -> Background {
    if let Some(bg) = std::env::var("COLORFGBG")
        .ok()
        .and_then(|v| background_from_colorfgbg(&v))
    {
        return bg;
    }
    if probe {
        if let Some(bg) = query_terminal(PROBE_TIMEOUT) {
            return bg;
        }
    }
    Background::Dark
}

/// Ask the terminal once and wait at most `timeout`. Runs before the key
/// loop takes stdin; a reply that lands after the deadline is an escape
/// sequence the key parser cannot read, which it drops.
#[cfg(unix)]
fn query_terminal(timeout: Duration) -> Option<Background> {
    use std::io::{Read, Write};
    use std::time::Instant;

    let mut out = std::io::stdout();
    out.write_all(QUERY).ok()?;
    out.flush().ok()?;

    let start = Instant::now();
    let mut buffer = Vec::new();
    loop {
        let left = timeout.checked_sub(start.elapsed())?;
        if !readable_within(libc::STDIN_FILENO, left) {
            return None;
        }
        let mut chunk = [0u8; 64];
        let n = std::io::stdin().lock().read(&mut chunk).ok()?;
        if n == 0 {
            return None;
        }
        buffer.extend_from_slice(&chunk[..n]);
        if let Some(reply) = take_reply(&buffer) {
            return background_from_reply(reply);
        }
    }
}

#[cfg(not(unix))]
fn query_terminal(_timeout: Duration) -> Option<Background> {
    None
}

/// `poll(2)` on one descriptor: true when a read would not block.
#[cfg(unix)]
fn readable_within(fd: i32, timeout: Duration) -> bool {
    let mut fds = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let millis = timeout.as_millis().min(i32::MAX as u128) as i32;
    // SAFETY: `fds` is a valid array of one initialized pollfd for the
    // duration of the call, and poll writes only into it.
    let ready = unsafe { libc::poll(&mut fds, 1, millis) };
    ready > 0 && fds.revents & libc::POLLIN != 0
}
