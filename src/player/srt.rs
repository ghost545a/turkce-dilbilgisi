//! SRT parsing and pairing of a Turkish and an English track
//! (ported from the desktop app's `subtitles.rs`).
//!
//! Both files are assumed to share timestamps, so cues are paired by order.

use serde::Serialize;

/// A single cue as read from one SRT file.
#[derive(Debug, Clone, PartialEq)]
pub struct RawCue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// A paired Turkish / English cue.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cue {
    pub id: usize,
    pub start: f64,
    pub end: f64,
    pub tr: String,
    pub en: String,
}

/// Result of merging the two tracks.
#[derive(Debug, Default)]
pub struct MergeResult {
    pub cues: Vec<Cue>,
    /// Non-fatal notes shown to the user (e.g. cue counts differ).
    pub warnings: Vec<String>,
}

/// Converts `HH:MM:SS,mmm` (or `HH:MM:SS.mmm`) into seconds.
pub fn parse_timestamp(s: &str) -> Result<f64, String> {
    let s = s.trim();
    let (hms, millis) = s
        .split_once([',', '.'])
        .ok_or_else(|| format!("Missing milliseconds in timestamp '{s}'"))?;
    let parts: Vec<&str> = hms.split(':').collect();
    if parts.len() != 3 {
        return Err(format!("Expected HH:MM:SS in timestamp '{s}'"));
    }
    let num = |v: &str, label: &str| -> Result<u32, String> {
        v.trim()
            .parse::<u32>()
            .map_err(|_| format!("Invalid {label} '{v}' in timestamp '{s}'"))
    };
    let hours = num(parts[0], "hours")?;
    let minutes = num(parts[1], "minutes")?;
    let seconds = num(parts[2], "seconds")?;
    let digits: String = millis.trim().chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return Err(format!("Invalid milliseconds in timestamp '{s}'"));
    }
    let padded = format!("{:0<3}", &digits[..digits.len().min(3)]);
    let ms = num(&padded, "milliseconds")?;
    Ok(hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds as f64 + ms as f64 / 1000.0)
}

fn parse_time_line(line: &str) -> Result<(f64, f64), String> {
    let (start, rest) = line
        .split_once("-->")
        .ok_or_else(|| format!("Not a timing line: '{line}'"))?;
    let end = rest.split_whitespace().next().unwrap_or("");
    Ok((parse_timestamp(start)?, parse_timestamp(end)?))
}

/// Removes simple formatting tags such as `<i>`, `</b>`, `<font ...>` and `{\an8}`.
fn strip_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let (mut angle, mut brace) = (false, false);
    for c in text.chars() {
        match c {
            '<' => angle = true,
            '>' if angle => angle = false,
            '{' => brace = true,
            '}' if brace => brace = false,
            _ if angle || brace => {}
            _ => out.push(c),
        }
    }
    out
}

/// Parses SRT text. Handles `\r\n`, a leading BOM, multi-line cues (joined by a
/// space), missing or extra blank lines and missing numeric indices. WebVTT
/// files also parse, because their cue lines use the same `-->` timing form.
pub fn parse(content: &str) -> Result<Vec<RawCue>, String> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut cues = Vec::new();
    let mut current: Option<(f64, f64)> = None;
    let mut lines: Vec<String> = Vec::new();

    let flush = |current: &mut Option<(f64, f64)>, lines: &mut Vec<String>, cues: &mut Vec<RawCue>| {
        if let Some((start, end)) = current.take() {
            let text = lines.join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
            cues.push(RawCue { start, end, text });
        }
        lines.clear();
    };

    for (n, raw) in content.lines().enumerate() {
        let line = raw.trim_end_matches('\r').trim();
        if line.contains("-->") {
            flush(&mut current, &mut lines, &mut cues);
            current = Some(parse_time_line(line).map_err(|e| format!("Line {}: {e}", n + 1))?);
        } else if line.is_empty() {
            flush(&mut current, &mut lines, &mut cues);
        } else if current.is_some() {
            lines.push(strip_tags(line));
        }
    }
    flush(&mut current, &mut lines, &mut cues);

    if cues.is_empty() {
        return Err("No subtitle cues found in file".to_string());
    }
    Ok(cues)
}

/// Pairs Turkish and English cues by order. The English track may be empty.
pub fn merge(target: &[RawCue], native: &[RawCue]) -> MergeResult {
    let mut warnings = Vec::new();
    if !native.is_empty() && target.len() != native.len() {
        warnings.push(format!(
            "Line counts differ (Turkish: {}, English: {}). Extra lines keep an empty translation.",
            target.len(),
            native.len()
        ));
    }
    let mut mismatched = 0usize;
    let cues = target
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let en = match native.get(i) {
                Some(n) => {
                    if (n.start - t.start).abs() > 0.05 {
                        mismatched += 1;
                    }
                    n.text.clone()
                }
                None => String::new(),
            };
            Cue { id: i + 1, start: t.start, end: t.end, tr: t.text.clone(), en }
        })
        .collect();
    if mismatched > 0 {
        warnings.push(format!("{mismatched} line(s) have start times that differ between the two files."));
    }
    MergeResult { cues, warnings }
}

#[derive(Serialize)]
struct Parsed {
    ok: bool,
    error: String,
    cues: Vec<Cue>,
    warnings: Vec<String>,
}

/// Parses both tracks and returns JSON `{ok, error, cues, warnings}`.
/// `native` may be empty (a Turkish-only lesson).
pub fn parse_dual_json(target: &str, native: &str) -> String {
    let result = match parse(target) {
        Err(e) => Parsed { ok: false, error: format!("Turkish subtitles: {e}"), cues: vec![], warnings: vec![] },
        Ok(t) => {
            let (n, mut warnings) = if native.trim().is_empty() {
                (Vec::new(), Vec::new())
            } else {
                match parse(native) {
                    Ok(n) => (n, Vec::new()),
                    Err(e) => (Vec::new(), vec![format!("English subtitles ignored: {e}")]),
                }
            };
            let merged = merge(&t, &n);
            warnings.extend(merged.warnings);
            Parsed { ok: true, error: String::new(), cues: merged.cues, warnings }
        }
    };
    serde_json::to_string(&result).unwrap_or_else(|_| "{\"ok\":false,\"error\":\"internal error\"}".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_math() {
        assert_eq!(parse_timestamp("00:01:24,500").unwrap(), 84.5);
        assert_eq!(parse_timestamp("01:00:00,000").unwrap(), 3600.0);
        assert_eq!(parse_timestamp("00:00:02.250").unwrap(), 2.25);
        assert!(parse_timestamp("garbage").is_err());
    }

    #[test]
    fn parses_crlf_bom_multiline_and_tags() {
        let srt = "\u{feff}1\r\n00:00:01,000 --> 00:00:03,500\r\n<i>Merhaba</i>\r\ndünya!\r\n\r\n\r\n2\r\n00:00:04,000 --> 00:00:06,000 X1:10\r\nNasılsın?\r\n";
        let cues = parse(srt).unwrap();
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].text, "Merhaba dünya!");
        assert_eq!((cues[0].start, cues[0].end), (1.0, 3.5));
        assert_eq!(cues[1].text, "Nasılsın?");
    }

    #[test]
    fn merges_by_order_and_warns() {
        let t = parse("1\n00:00:01,000 --> 00:00:02,000\nMerhaba\n\n2\n00:00:03,000 --> 00:00:04,000\nEvet\n").unwrap();
        let n = parse("1\n00:00:01,000 --> 00:00:02,000\nHello\n").unwrap();
        let r = merge(&t, &n);
        assert_eq!(r.cues[0].tr, "Merhaba");
        assert_eq!(r.cues[0].en, "Hello");
        assert_eq!(r.cues[1].en, "");
        assert_eq!(r.warnings.len(), 1);
        assert!(merge(&t, &[]).warnings.is_empty(), "a Turkish-only lesson is not a warning");
    }

    #[test]
    fn json_reports_errors_and_reads_the_sample_files() {
        assert!(parse_dual_json("not subtitles", "").contains("\"ok\":false"));
        let tr = include_str!("../../player/samples/sample_tr.srt");
        let en = include_str!("../../player/samples/sample_en.srt");
        let good = parse_dual_json(tr, en);
        assert!(good.contains("\"ok\":true"));
        assert!(good.contains("Kahve içmek ister misin?") && good.contains("Would you like"));
        assert!(good.contains("\"warnings\":[]"));
    }
}
