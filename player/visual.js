// Visuals for audio-only lessons: a live ring of frequency bars in the video
// area and a waveform strip with the subtitle lines marked on it.
//
// Both are computed from the decoded audio at the current play position (a
// small FFT per frame) rather than by routing the playing sound through Web
// Audio, so cross-origin audio is never muted and seeking just works.
import { RATE, decodeAudio } from './audio-utils.js';

const FFT_SIZE = 1024;
const BARS = 48;
const PEAK_BUCKETS = 8192;

// ───────────────────────────── FFT ─────────────────────────────

const hann = new Float32Array(FFT_SIZE).map((_, i) => 0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (FFT_SIZE - 1)));
const bitrev = (() => {
  const bits = Math.log2(FFT_SIZE);
  const out = new Uint32Array(FFT_SIZE);
  for (let i = 0; i < FFT_SIZE; i++) {
    let r = 0;
    for (let b = 0; b < bits; b++) r |= ((i >> b) & 1) << (bits - 1 - b);
    out[i] = r;
  }
  return out;
})();
const re = new Float32Array(FFT_SIZE), im = new Float32Array(FFT_SIZE);

/** Magnitude spectrum (first half) of `FFT_SIZE` samples starting at `start`. */
function spectrum(samples, start, out) {
  for (let i = 0; i < FFT_SIZE; i++) {
    const s = samples[start + i] || 0;
    re[bitrev[i]] = s * hann[i];
    im[bitrev[i]] = 0;
  }
  for (let size = 2; size <= FFT_SIZE; size <<= 1) {
    const half = size >> 1, step = (-2 * Math.PI) / size;
    for (let i = 0; i < FFT_SIZE; i += size) {
      for (let k = 0; k < half; k++) {
        const a = step * k, wr = Math.cos(a), wi = Math.sin(a);
        const j = i + k, l = j + half;
        const tr = re[l] * wr - im[l] * wi, ti = re[l] * wi + im[l] * wr;
        re[l] = re[j] - tr; im[l] = im[j] - ti;
        re[j] += tr; im[j] += ti;
      }
    }
  }
  for (let i = 0; i < out.length; i++) out[i] = Math.hypot(re[i], im[i]);
  return out;
}

// Log-spaced band edges from 80 Hz to 6 kHz, as FFT bin indices.
const bandEdges = Array.from({ length: BARS + 1 }, (_, i) => {
  const hz = 80 * Math.pow(6000 / 80, i / BARS);
  return Math.max(1, Math.round((hz / RATE) * FFT_SIZE));
});

function fmtTime(s) {
  const t = Math.max(0, Math.floor(s));
  return `${String(Math.floor(t / 60)).padStart(2, '0')}:${String(t % 60).padStart(2, '0')}`;
}

// ───────────────────────────── AudioVisual ─────────────────────────────

export class AudioVisual {
  /**
   * @param {object} o
   * @param {HTMLCanvasElement} o.live   canvas inside the media box
   * @param {HTMLElement} o.note         note shown when the audio cannot be analysed
   * @param {HTMLElement} o.strip        waveform container (focusable)
   * @param {HTMLCanvasElement} o.stripCanvas
   * @param {HTMLElement} o.tip          hover time label
   * @param {() => number} o.getTime
   * @param {() => boolean} o.isPaused
   * @param {(t: number) => void} o.seek
   */
  constructor(o) {
    Object.assign(this, o);
    this.samples = null;
    this.peaks = null;
    this.cues = [];
    this.active = -1;
    this.token = 0;
    this.running = false;
    this.levels = new Float32Array(BARS);
    this.mags = new Float32Array(FFT_SIZE / 2);
    this.reduced = matchMedia('(prefers-reduced-motion: reduce)');
    this.dragging = false;
    this.lastStrip = '';
    this.bindStrip();
    new ResizeObserver(() => { this.lastStrip = ''; this.sizeCanvases(); }).observe(this.strip);
    new ResizeObserver(() => { this.sizeCanvases(); this.wake(); }).observe(this.live);
  }

  /** Starts the visuals for an audio-only source. */
  async load(source) {
    const token = ++this.token;
    this.samples = null;
    this.peaks = null;
    this.live.hidden = false;
    this.note.hidden = true;
    this.strip.hidden = true;
    this.start();
    try {
      const samples = await decodeAudio(source);
      if (token !== this.token) return;
      this.samples = samples;
      this.peaks = computePeaks(samples);
      this.strip.hidden = false;
      this.lastStrip = '';
      this.sizeCanvases();
    } catch (e) {
      if (token !== this.token) return;
      this.note.textContent = e.code === 'cors'
        ? 'The waveform is not available for this link: its host does not let other websites read the sound. Choose the file from your device to see it.'
        : `The waveform is not available: ${e.message}`;
      this.note.hidden = false;
    }
  }

  /** Hides everything (video, YouTube or no lesson). */
  clear() {
    this.token++;
    this.samples = null;
    this.peaks = null;
    this.live.hidden = true;
    this.note.hidden = true;
    this.strip.hidden = true;
    this.running = false;
  }

  setCues(cues) { this.cues = cues || []; this.lastStrip = ''; }
  setActive(i) { this.active = i; }

  duration() { return this.samples ? this.samples.length / RATE : 0; }

  sizeCanvases() {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    for (const c of [this.live, this.stripCanvas]) {
      const w = Math.round(c.clientWidth * dpr), h = Math.round(c.clientHeight * dpr);
      if (w && h && (c.width !== w || c.height !== h)) { c.width = w; c.height = h; }
    }
    this.lastStrip = '';
  }

  /** Runs the drawing loop while playing; it sleeps once paused and settled. */
  start() {
    if (this.running) return;
    this.running = true;
    let quiet = 0;
    const loop = (now) => {
      if (!this.running) return;
      this.drawLive(now / 1000);
      this.drawStrip();
      const settled = this.levels.every((v) => Math.abs(v - 0.1) < 0.002);
      quiet = this.isPaused() && settled && !this.dragging ? quiet + 1 : 0;
      if (quiet > 2) { this.running = false; return; }
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
  }

  /** Redraws after play, pause, seeking or a resize. */
  wake() {
    if (!this.live.hidden) { this.lastStrip = ''; this.start(); }
  }

  // ── live ring ──

  drawLive(now) {
    const c = this.live, ctx = c.getContext('2d');
    const w = c.width, h = c.height;
    if (!w || !h) return;
    const playing = !this.isPaused();
    const still = this.reduced.matches;

    let energy = 0;
    if (this.samples && playing && !still) {
      const at = Math.floor(this.getTime() * RATE) - FFT_SIZE / 2;
      spectrum(this.samples, Math.max(0, at), this.mags);
      for (let b = 0; b < BARS; b++) {
        let m = 0;
        for (let k = bandEdges[b]; k <= Math.max(bandEdges[b], bandEdges[b + 1] - 1); k++) m = Math.max(m, this.mags[k] || 0);
        const db = 20 * Math.log10(m + 1e-9);
        const v = Math.max(0, Math.min(1, (db + 30) / 50));
        this.levels[b] = Math.max(v, this.levels[b] * 0.86);
        energy += this.levels[b];
      }
      energy /= BARS;
    } else {
      // Paused: settle to a still ring (no slow looping motion).
      for (let b = 0; b < BARS; b++) {
        this.levels[b] += (0.1 - this.levels[b]) * (still ? 1 : 0.12);
        energy += this.levels[b];
      }
      energy /= BARS;
    }

    ctx.clearRect(0, 0, w, h);
    const bg = ctx.createRadialGradient(w / 2, h / 2, 0, w / 2, h / 2, Math.max(w, h) * 0.7);
    bg.addColorStop(0, '#2a0b13');
    bg.addColorStop(1, '#07070b');
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, w, h);

    const cx = w / 2, cy = h * 0.46, unit = Math.min(w, h);
    const r0 = unit * 0.17, len = unit * 0.2;
    ctx.save();
    ctx.translate(cx, cy);
    ctx.lineCap = 'round';
    ctx.lineWidth = Math.max(2, unit * 0.011);
    const total = BARS * 2;
    for (let i = 0; i < total; i++) {
      // Mirror the bands so low frequencies sit at the top and bottom.
      const band = i < BARS ? i : total - 1 - i;
      const v = this.levels[band];
      const a = (i / total) * Math.PI * 2 - Math.PI / 2;
      const r1 = r0 + 4, r2 = r0 + 6 + v * len;
      ctx.strokeStyle = `rgba(${220 + v * 35 | 0}, ${20 + v * 90 | 0}, ${60 + v * 60 | 0}, ${0.35 + v * 0.65})`;
      ctx.beginPath();
      ctx.moveTo(Math.cos(a) * r1, Math.sin(a) * r1);
      ctx.lineTo(Math.cos(a) * r2, Math.sin(a) * r2);
      ctx.stroke();
    }
    const glow = r0 * (0.86 + energy * 0.35);
    const core = ctx.createRadialGradient(-glow * 0.25, -glow * 0.3, glow * 0.1, 0, 0, glow);
    core.addColorStop(0, '#E8284A');
    core.addColorStop(0.6, '#9B111E');
    core.addColorStop(1, '#3a0610');
    ctx.shadowColor = 'rgba(220, 20, 60, 0.7)';
    ctx.shadowBlur = unit * (0.05 + energy * 0.12);
    ctx.fillStyle = core;
    ctx.beginPath();
    ctx.arc(0, 0, glow, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
  }

  // ── waveform strip ──

  drawStrip() {
    if (this.strip.hidden || !this.peaks) return;
    const c = this.stripCanvas, ctx = c.getContext('2d');
    const w = c.width, h = c.height;
    if (!w || !h) return;
    const dur = this.duration();
    const t = Math.min(this.getTime(), dur);
    const key = `${w}x${h}:${Math.round((t / dur) * w)}:${this.active}:${this.hover ?? ''}`;
    if (key === this.lastStrip) return;
    this.lastStrip = key;

    ctx.clearRect(0, 0, w, h);
    const x = (sec) => (sec / dur) * w;

    // Subtitle lines as bands.
    this.cues.forEach((cue, i) => {
      const x0 = x(cue.start), x1 = Math.max(x0 + 1, x(cue.end));
      ctx.fillStyle = i === this.active ? 'rgba(220, 20, 60, 0.2)' : 'rgba(255, 255, 255, 0.05)';
      ctx.fillRect(x0, 0, x1 - x0, h);
      ctx.fillStyle = i === this.active ? 'rgba(220, 20, 60, 0.85)' : 'rgba(255, 255, 255, 0.18)';
      ctx.fillRect(x0, 0, Math.max(1, x1 - x0), Math.max(2, h * 0.04));
    });

    // Waveform, mirrored around the middle.
    const { min, max, peak } = this.peaks, n = min.length;
    const mid = h / 2, scale = (h * 0.44) / (peak || 1);
    const played = x(t);
    const bar = Math.max(1, Math.round(w / 600));
    for (let px = 0; px < w; px += bar + 1) {
      const a = Math.floor((px / w) * n), b = Math.max(a + 1, Math.floor(((px + bar) / w) * n));
      let lo = 0, hi = 0;
      for (let k = a; k < b && k < n; k++) { if (min[k] < lo) lo = min[k]; if (max[k] > hi) hi = max[k]; }
      const top = mid - Math.max(1, hi * scale), bottom = mid - Math.min(-1, lo * scale);
      ctx.fillStyle = px < played ? '#DC143C' : 'rgba(170, 170, 190, 0.45)';
      ctx.fillRect(px, top, bar, bottom - top);
    }

    // Play head and hover marker.
    ctx.fillStyle = '#fff';
    ctx.fillRect(Math.round(played) - 1, 0, 2, h);
    if (this.hover !== undefined && this.hover !== null) {
      ctx.fillStyle = 'rgba(255, 255, 255, 0.35)';
      ctx.fillRect(Math.round(this.hover), 0, 1, h);
    }
    this.strip.setAttribute('aria-valuenow', String(Math.round(t)));
    this.strip.setAttribute('aria-valuemax', String(Math.round(dur)));
    this.strip.setAttribute('aria-valuetext', `${fmtTime(t)} of ${fmtTime(dur)}`);
  }

  bindStrip() {
    const posToTime = (e) => {
      const r = this.stripCanvas.getBoundingClientRect();
      const f = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width));
      return { f, t: f * this.duration(), px: f * this.stripCanvas.width, x: e.clientX - r.left };
    };
    const showTip = (p) => {
      this.tip.hidden = false;
      this.tip.textContent = fmtTime(p.t);
      this.tip.style.left = `${p.x}px`;
    };
    this.strip.addEventListener('pointerdown', (e) => {
      if (!this.samples) return;
      this.dragging = true;
      this.strip.setPointerCapture(e.pointerId);
      const p = posToTime(e);
      this.seek(p.t);
      showTip(p);
    });
    this.strip.addEventListener('pointermove', (e) => {
      if (!this.samples) return;
      const p = posToTime(e);
      this.hover = p.px;
      showTip(p);
      if (this.dragging) this.seek(p.t);
    });
    const end = (e) => {
      this.dragging = false;
      if (this.strip.hasPointerCapture?.(e.pointerId)) this.strip.releasePointerCapture(e.pointerId);
      const r = this.strip.getBoundingClientRect();
      const inside = e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom;
      if (!inside || e.pointerType === 'touch') { this.hover = null; this.tip.hidden = true; this.lastStrip = ''; }
    };
    this.strip.addEventListener('pointerup', end);
    this.strip.addEventListener('pointercancel', end);
    this.strip.addEventListener('pointerleave', () => { if (!this.dragging) { this.hover = null; this.tip.hidden = true; } });
  }
}

/** Min / max of the audio in `PEAK_BUCKETS` equal slices, plus the overall peak. */
function computePeaks(samples) {
  const n = Math.min(PEAK_BUCKETS, samples.length);
  const min = new Float32Array(n), max = new Float32Array(n);
  let peak = 0;
  for (let b = 0; b < n; b++) {
    const a = Math.floor((b / n) * samples.length), z = Math.floor(((b + 1) / n) * samples.length);
    let lo = 0, hi = 0;
    for (let i = a; i < z; i++) { const s = samples[i]; if (s < lo) lo = s; else if (s > hi) hi = s; }
    min[b] = lo; max[b] = hi;
    peak = Math.max(peak, hi, -lo);
  }
  return { min, max, peak };
}
