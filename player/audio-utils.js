// Audio helpers shared by the page and the speech worker: splitting at quiet
// moments, WAV encoding, cleaning Whisper output and writing SRT files.

export const RATE = 16000;

/** Root-mean-square energy of each `frameSec` frame. */
function frameEnergy(samples, rate, frameSec = 0.02) {
  const size = Math.max(1, Math.round(rate * frameSec));
  const out = new Float32Array(Math.ceil(samples.length / size));
  for (let f = 0; f < out.length; f++) {
    let sum = 0;
    const end = Math.min(samples.length, (f + 1) * size);
    for (let i = f * size; i < end; i++) sum += samples[i] * samples[i];
    out[f] = Math.sqrt(sum / Math.max(1, end - f * size));
  }
  return { energy: out, size };
}

/**
 * Splits audio into blocks of at most `maxSec`, cutting each block at the
 * quietest moment within the last `windowSec` so words are not cut in half.
 * Returns `[{start, end}]` in samples.
 */
export function splitAtQuiet(samples, rate, maxSec, windowSec = 5) {
  const { energy, size } = frameEnergy(samples, rate);
  const blocks = [];
  let start = 0;
  const max = Math.round(maxSec * rate);
  while (samples.length - start > max) {
    const hi = Math.floor((start + max) / size);
    const lo = Math.max(Math.floor(start / size) + 1, hi - Math.round(windowSec * rate / size));
    let best = hi;
    for (let f = lo; f <= hi && f < energy.length; f++) if (energy[f] < energy[best]) best = f;
    const cut = Math.min(samples.length, best * size);
    blocks.push({ start, end: cut });
    start = cut;
  }
  if (samples.length > start) blocks.push({ start, end: samples.length });
  return blocks;
}

/**
 * Finds stretches of speech separated by pauses of at least `minPauseSec`,
 * each at most `maxSec` long. Used for providers that return text without
 * timestamps: each stretch becomes one subtitle line.
 */
export function speechPieces(samples, rate, maxSec = 12, minPauseSec = 0.35) {
  const { energy, size } = frameEnergy(samples, rate);
  if (!energy.length) return [];
  const sorted = Float32Array.from(energy).sort();
  const noise = sorted[Math.floor(sorted.length * 0.1)];
  const peak = sorted[Math.floor(sorted.length * 0.95)];
  const threshold = noise + (peak - noise) * 0.12;
  const pauseFrames = Math.round(minPauseSec * rate / size);
  const pieces = [];
  let begin = -1, quiet = 0;
  for (let f = 0; f <= energy.length; f++) {
    const loud = f < energy.length && energy[f] > threshold;
    if (loud) {
      if (begin < 0) begin = f;
      quiet = 0;
    } else if (begin >= 0) {
      quiet++;
      if (quiet >= pauseFrames || f === energy.length) {
        pieces.push({ start: begin * size, end: Math.min(samples.length, (f - quiet + 1) * size) });
        begin = -1; quiet = 0;
      }
    }
  }
  // Long stretches are split again at their quietest points.
  const out = [];
  for (const p of pieces) {
    const pad = Math.round(0.15 * rate);
    const s = Math.max(0, p.start - pad), e = Math.min(samples.length, p.end + pad);
    if ((e - s) / rate <= maxSec) { out.push({ start: s, end: e }); continue; }
    for (const b of splitAtQuiet(samples.subarray(s, e), rate, maxSec, Math.min(4, maxSec / 3))) {
      out.push({ start: s + b.start, end: s + b.end });
    }
  }
  return out.filter((p) => p.end - p.start > rate * 0.3);
}

/** 16-bit PCM mono WAV. */
export function encodeWav(samples, rate = RATE) {
  const buf = new ArrayBuffer(44 + samples.length * 2);
  const v = new DataView(buf);
  const str = (o, s) => { for (let i = 0; i < s.length; i++) v.setUint8(o + i, s.charCodeAt(i)); };
  str(0, 'RIFF'); v.setUint32(4, 36 + samples.length * 2, true); str(8, 'WAVE');
  str(12, 'fmt '); v.setUint32(16, 16, true); v.setUint16(20, 1, true); v.setUint16(22, 1, true);
  v.setUint32(24, rate, true); v.setUint32(28, rate * 2, true); v.setUint16(32, 2, true); v.setUint16(34, 16, true);
  str(36, 'data'); v.setUint32(40, samples.length * 2, true);
  for (let i = 0; i < samples.length; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    v.setInt16(44 + i * 2, s < 0 ? s * 0x8000 : s * 0x7fff, true);
  }
  return new Blob([buf], { type: 'audio/wav' });
}

// Phrases Whisper is known to invent over silence or music in Turkish audio.
const HALLUCINATION = /^(altyaz[ıi]\b.*|.*abone ol.*|izledi[ğg]iniz i[çc]in te[şs]ekk[üu]r ederim\.?|\[.*\]|\(.*\)|♪+|\.+|-+)$/i;

/**
 * Cleans timed segments: trims text, drops empty and invented lines, repairs
 * missing or overlapping times, and splits long segments at sentence ends.
 */
export function tidySegments(segments, totalSec) {
  const out = [];
  for (const seg of segments) {
    const text = (seg.text || '').replace(/\s+/g, ' ').trim();
    if (!text || HALLUCINATION.test(text)) continue;
    let start = Number.isFinite(seg.start) ? seg.start : (out.at(-1)?.end ?? 0);
    let end = Number.isFinite(seg.end) && seg.end > start ? seg.end : Math.min(totalSec, start + Math.max(1.5, text.length / 14));
    if (out.length && start < out.at(-1).end) start = out.at(-1).end;
    if (end <= start) end = start + 0.5;
    // Split long segments at sentence ends, sharing time by character count.
    const parts = end - start > 8 ? text.split(/(?<=[.!?…])\s+(?=\S)/) : [text];
    if (parts.length > 1) {
      const total = parts.reduce((n, p) => n + p.length, 0);
      let t = start;
      for (const p of parts) {
        const d = (end - start) * (p.length / total);
        out.push({ start: t, end: t + d, text: p });
        t += d;
      }
    } else {
      out.push({ start, end, text });
    }
  }
  // Drop an immediately repeated line (a Whisper loop).
  return out.filter((s, i) => i === 0 || s.text !== out[i - 1].text);
}

function srtTime(sec) {
  const ms = Math.max(0, Math.round(sec * 1000));
  const h = Math.floor(ms / 3600000), m = Math.floor((ms % 3600000) / 60000);
  const s = Math.floor((ms % 60000) / 1000), r = ms % 1000;
  const p = (n, w = 2) => String(n).padStart(w, '0');
  return `${p(h)}:${p(m)}:${p(s)},${p(r, 3)}`;
}

/** SRT text for one field (`tr` or `en`) of `[{start, end, tr, en}]`. */
export function toSrt(cues, field) {
  return cues.map((c, i) => `${i + 1}\n${srtTime(c.start)} --> ${srtTime(c.end)}\n${c[field] || ''}\n`).join('\n');
}

// ───────────────────────────── decoding ─────────────────────────────

const decodeCache = { key: null, promise: null };

/**
 * Decodes the audio track of a lesson source (`{kind: 'file', file}` or
 * `{kind: 'url', url}`) to 16 kHz mono. The result for the open lesson is
 * cached, so the waveform view and Auto subtitles share one decode. Treat the
 * returned array as read-only.
 */
export function decodeAudio(src, signal) {
  const key = src.key || src.url || (src.file && src.file.name);
  if (decodeCache.key === key && decodeCache.promise) return decodeCache.promise;
  const promise = decodeUncached(src, signal);
  decodeCache.key = key;
  decodeCache.promise = promise;
  promise.catch(() => { if (decodeCache.promise === promise) { decodeCache.key = null; decodeCache.promise = null; } });
  return promise;
}

async function decodeUncached(src, signal) {
  let bytes;
  if (src.kind === 'file') {
    bytes = await src.file.arrayBuffer();
  } else {
    let res;
    try { res = await fetch(src.url, { signal }); } catch (e) {
      if (signal && signal.aborted) throw e;
      const err = new Error('The site hosting this file does not let other websites read it (CORS), so its sound cannot be analysed here. Download it and choose it as a file instead.');
      err.code = 'cors';
      throw err;
    }
    if (!res.ok) throw new Error(`Could not download the file for analysis (HTTP ${res.status}).`);
    bytes = await res.arrayBuffer();
  }
  if (signal && signal.aborted) throw new DOMException('Cancelled', 'AbortError');
  const Ctx = window.AudioContext || window.webkitAudioContext;
  const ac = new Ctx({ sampleRate: RATE });
  let decoded;
  try {
    decoded = await ac.decodeAudioData(bytes);
  } catch {
    throw new Error('The browser could not read the audio track of this file. Try an .mp4, .webm, .mp3, .ogg or .wav file.');
  } finally {
    ac.close();
  }
  if (decoded.numberOfChannels === 1) return decoded.getChannelData(0).slice();
  const out = new Float32Array(decoded.length);
  for (let c = 0; c < decoded.numberOfChannels; c++) {
    const ch = decoded.getChannelData(c);
    for (let i = 0; i < out.length; i++) out[i] += ch[i] / decoded.numberOfChannels;
  }
  return out;
}
