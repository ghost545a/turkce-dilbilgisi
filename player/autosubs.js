// Automatic subtitles: Turkish speech → timed Turkish lines → English.
// Runs on the visitor's device (Whisper + opus-mt or Chrome's translator) or,
// with the visitor's own API key, through OpenAI, Groq or Qwen.
import { RATE, splitAtQuiet, speechPieces, encodeWav, tidySegments, toSrt, decodeAudio } from './audio-utils.js';

const $ = (id) => document.getElementById(id);
const API_KEY = 'trlearn.api';

const PROVIDERS = {
  openai: {
    name: 'OpenAI', base: 'https://api.openai.com/v1', asr: 'whisper-1', chat: 'gpt-4o-mini', kind: 'whisper',
  },
  groq: {
    name: 'Groq', base: 'https://api.groq.com/openai/v1', asr: 'whisper-large-v3', chat: 'llama-3.3-70b-versatile', kind: 'whisper',
  },
  qwen: {
    name: 'Qwen (Alibaba Model Studio)', base: 'https://dashscope-intl.aliyuncs.com/compatible-mode/v1',
    asr: 'qwen3-asr-flash', chat: 'qwen-plus', kind: 'qwen',
  },
};

function readSettings() {
  try { return JSON.parse(localStorage.getItem(API_KEY)) || {}; } catch { return {}; }
}
function writeSettings(s) {
  try { localStorage.setItem(API_KEY, JSON.stringify(s)); } catch { /* storage blocked: settings last for this visit */ }
}

class Cancelled extends Error {}

export function setupAutoSubs(ctx) {
  let job = null; // { cancel(), cues }
  let result = null;
  const settings = readSettings();

  // ── settings UI ──
  const provider = $('api-provider'), key = $('api-key'), chat = $('api-chat-model'), remember = $('api-remember');
  provider.value = PROVIDERS[settings.provider] ? settings.provider : 'openai';
  const models = settings.models || {};
  const syncProvider = () => {
    chat.value = models[provider.value] || PROVIDERS[provider.value].chat;
    key.value = (settings.keys || {})[provider.value] || '';
    remember.checked = Boolean(key.value);
  };
  syncProvider();
  provider.addEventListener('change', syncProvider);
  $('api-forget').addEventListener('click', () => {
    settings.keys = {};
    writeSettings(settings);
    key.value = '';
    remember.checked = false;
    ctx.toast('Saved API keys removed from this browser.');
  });
  document.querySelectorAll('input[name="auto-mode"]').forEach((r) => r.addEventListener('change', () => {
    const api = mode() === 'api';
    $('auto-api').hidden = !api;
    $('auto-device').hidden = api;
  }));
  const mode = () => document.querySelector('input[name="auto-mode"]:checked').value;

  const hasGpu = 'gpu' in navigator;
  if (!hasGpu) {
    const turbo = document.querySelector('#asr-model option[value="turbo"]');
    if (turbo) { turbo.disabled = true; turbo.textContent += ' — not available in this browser'; }
  }
  $('device-info').textContent = hasGpu
    ? 'Your browser supports WebGPU, so the speech model can run on your graphics card.'
    : 'Your browser has no WebGPU, so the speech model runs on the processor. Expect several minutes per minute of video with the larger models.';

  // ── dialog ──
  function blockedReason() {
    const src = ctx.getSource();
    if (!src) return 'Open a video first (a file from your device or a direct link to a media file).';
    if (src.kind === 'youtube') return 'YouTube does not let other websites read the sound of its videos, so subtitles cannot be created automatically for YouTube links. Add .srt files instead, or open the video as a file you have the rights to use.';
    return '';
  }
  function refreshDialog() {
    const reason = blockedReason();
    $('auto-blocked').hidden = !reason;
    $('auto-blocked').textContent = reason;
    $('auto-start').disabled = Boolean(reason) || Boolean(job);
  }
  $('btn-auto').addEventListener('click', () => {
    refreshDialog();
    ctx.openModal('auto-dialog');
  });
  document.addEventListener('lesson-media', () => {
    result = null;
    $('auto-done').hidden = true;
    refreshDialog();
  });

  function setProgress(fraction, text) {
    $('auto-progress').hidden = false;
    $('auto-bar').style.width = `${Math.round(Math.max(0, Math.min(1, fraction)) * 100)}%`;
    if (text) $('auto-status').textContent = text;
  }

  function finish(error) {
    job = null;
    $('auto-cancel').hidden = true;
    $('auto-start').hidden = false;
    refreshDialog();
    if (error) {
      $('auto-progress').hidden = true;
      if (!(error instanceof Cancelled)) $('auto-error').textContent = error.message || String(error);
    }
  }

  $('auto-start').addEventListener('click', async () => {
    if (job) return;
    $('auto-error').textContent = '';
    $('auto-done').hidden = true;
    const src = ctx.getSource();
    const runMode = mode();

    let api = null;
    if (runMode === 'api') {
      const p = provider.value;
      if (!key.value.trim()) { $('auto-error').textContent = 'Paste your API key first.'; return; }
      api = { ...PROVIDERS[p], id: p, key: key.value.trim(), chat: chat.value.trim() || PROVIDERS[p].chat };
      settings.provider = p;
      settings.models = { ...(settings.models || {}), [p]: api.chat };
      settings.keys = { ...(settings.keys || {}) };
      if (remember.checked) settings.keys[p] = api.key; else delete settings.keys[p];
      writeSettings(settings);
    }

    const abort = new AbortController();
    let worker = null, speechWorker = null, translator = null;
    job = {
      cancel() {
        abort.abort();
        worker?.terminate();
        speechWorker?.terminate();
        translator?.destroy?.();
      },
    };
    $('auto-start').hidden = true;
    $('auto-cancel').hidden = false;

    // Chrome's on-device translator must be created while the click still counts
    // as a user action, so it is requested before any long work starts.
    // Some browsers expose the API but never answer; those fall back to the
    // downloadable model after a time limit.
    let translatorReady = null;
    let translatorProgress = null;
    if (runMode === 'device' && 'Translator' in self) {
      translatorReady = (async () => {
        const opts = { sourceLanguage: 'tr', targetLanguage: 'en' };
        const availability = await withTimeout(self.Translator.availability(opts), 5000);
        if (!availability || availability === 'unavailable') return null;
        translator = await self.Translator.create({
          ...opts,
          monitor(m) { m.addEventListener('downloadprogress', (e) => { translatorProgress = e.loaded; }); },
        });
        return translator;
      })().catch(() => null);
    }

    try {
      setProgress(0.01, 'Reading the audio of the video…');
      const audio = await decodeAudio(src, abort.signal);
      const totalSec = audio.length / RATE;
      if (totalSec < 0.5) throw new Error('This video has no audio to transcribe.');

      let segments, english;
      if (runMode === 'device') {
        const model = $('asr-model').value;
        if (model.startsWith('seda')) {
          // Turkish-only seda models run in sherpa-onnx (a classic worker);
          // translation still uses the Transformers.js worker.
          speechWorker = new Worker(new URL('./seda-worker.js', import.meta.url));
          segments = await sedaTranscribe(speechWorker, audio, model.replace('seda-', ''), setProgress);
          speechWorker.terminate();
          speechWorker = null;
          worker = new Worker(new URL('./asr-worker.js', import.meta.url), { type: 'module' });
        } else {
          worker = new Worker(new URL('./asr-worker.js', import.meta.url), { type: 'module' });
          segments = await deviceTranscribe(worker, audio, model, hasGpu, setProgress);
        }
        segments = tidySegments(segments, totalSec);
        if (!segments.length) throw new Error('No Turkish speech was recognised in this video.');
        const lines = segments.map((s) => s.text);
        let t = null;
        if (translatorReady) {
          setProgress(0.8, "Preparing Chrome's built-in Turkish–English translator…");
          const ticker = setInterval(() => {
            if (translatorProgress !== null) setProgress(0.8, `Downloading Chrome's Turkish–English translator… ${Math.round(translatorProgress * 100)}%`);
          }, 500);
          // Give a running language-pack download longer than a browser that never answers.
          t = await withTimeout(translatorReady, 20000).then((v) => v ?? (translatorProgress !== null ? withTimeout(translatorReady, 180000) : null));
          clearInterval(ticker);
        }
        english = t
          ? await chromeTranslate(t, lines, setProgress, abort.signal)
          : await deviceTranslate(worker, lines, setProgress);
      } else {
        segments = tidySegments(await apiTranscribe(api, audio, setProgress, abort.signal), totalSec);
        if (!segments.length) throw new Error('No Turkish speech was recognised in this video.');
        english = await apiTranslate(api, segments.map((s) => s.text), setProgress, abort.signal);
      }

      const cues = segments.map((s, i) => ({ id: i + 1, start: s.start, end: s.end, tr: s.text, en: english[i] || '' }));
      result = { cues, name: (src.name || 'lesson').replace(/\.[^.]+$/, '') };
      ctx.setCues(cues);
      ctx.onSubLinksCleared();
      setProgress(1, 'Done.');
      $('auto-summary').textContent = `Created ${cues.length} subtitle line${cues.length === 1 ? '' : 's'} for ${fmtDuration(totalSec)} of audio. They are loaded in the player now; download them to keep them.`;
      $('auto-done').hidden = false;
      finish();
    } catch (e) {
      finish(abort.signal.aborted ? new Cancelled() : e);
    } finally {
      worker?.terminate();
      speechWorker?.terminate();
      translator?.destroy?.();
    }
  });

  $('auto-cancel').addEventListener('click', () => {
    if (!job) return;
    job.cancel();
    $('auto-status').textContent = 'Cancelled.';
    finish(new Cancelled());
  });

  $('dl-tr').addEventListener('click', () => result && ctx.download(`${result.name}_tr.srt`, toSrt(result.cues, 'tr')));
  $('dl-en').addEventListener('click', () => result && ctx.download(`${result.name}_en.srt`, toSrt(result.cues, 'en')));
}

/** Resolves to the promise's value, or `null` if it takes longer than `ms`. */
function withTimeout(promise, ms) {
  return Promise.race([promise, new Promise((resolve) => setTimeout(() => resolve(null), ms))]);
}

function fmtDuration(sec) {
  const m = Math.floor(sec / 60), s = Math.round(sec % 60);
  return m ? `${m} min ${s} s` : `${s} s`;
}

// ───────────────────────────── on device ─────────────────────────────

function workerCall(worker, message, transfer, onMessage) {
  return new Promise((resolve, reject) => {
    worker.onmessage = ({ data }) => {
      if (data.type === 'error') reject(new Error(friendlyModelError(data.message)));
      else if (data.type.endsWith('-result')) resolve(data);
      else onMessage(data);
    };
    worker.onerror = (e) => reject(new Error(`The speech engine stopped: ${e.message || 'unknown error'}`));
    worker.postMessage(message, transfer);
  });
}

function friendlyModelError(msg) {
  if (/fetch|network|Failed to load/i.test(msg)) return `Could not download the model (${msg}). Check your connection and try again.`;
  if (/memory|allocation|OOM/i.test(msg)) return `Not enough memory for this model (${msg}). Try a smaller model.`;
  return msg;
}

function downloadTracker(setProgress, from, to, label) {
  const files = new Map();
  return (d) => {
    if (d.done) { const f = files.get(d.file); if (f) f.loaded = f.total; }
    else files.set(d.file, { loaded: d.loaded, total: d.total });
    let loaded = 0, total = 0;
    for (const f of files.values()) { loaded += f.loaded; total += f.total; }
    if (total) setProgress(from + (to - from) * (loaded / total), `${label} ${(loaded / 1048576).toFixed(0)} / ${(total / 1048576).toFixed(0)} MB (only the first time)`);
  };
}

async function gpuHasF16() {
  try { const a = await navigator.gpu?.requestAdapter(); return Boolean(a && a.features.has('shader-f16')); } catch { return false; }
}

async function deviceTranscribe(worker, audio, size, webgpu, setProgress) {
  const f16 = webgpu && size === 'turbo' ? await gpuHasF16() : false;
  const onDownload = downloadTracker(setProgress, 0.02, 0.25, 'Downloading the speech model…');
  setProgress(0.02, 'Loading the speech model…');
  const copy = audio.slice(); // the decoded audio is cached and shared, so the worker gets a copy
  const result = await workerCall(worker, { type: 'asr', audio: copy, size, webgpu, f16 }, [copy.buffer], (d) => {
    if (d.type === 'download' && d.stage === 'asr') onDownload(d);
    else if (d.type === 'note') setProgress(0.25, d.text);
    else if (d.type === 'asr') setProgress(0.25 + 0.55 * (d.done / d.total), `Listening… part ${Math.min(d.done + 1, d.total)} of ${d.total}${d.partial ? ` · ${d.partial} lines so far` : ''}`);
  });
  return result.segments;
}

async function sedaTranscribe(worker, audio, size, setProgress) {
  const onDownload = downloadTracker(setProgress, 0.02, 0.2, 'Downloading the Turkish speech model…');
  setProgress(0.02, 'Loading the Turkish speech model…');
  // seda returns text without timings, so each stretch of speech between
  // pauses is decoded on its own and becomes one subtitle line.
  // Join stretches separated by short pauses into lines of up to 8 s, so
  // each line reads (and translates) as a phrase rather than a fragment.
  const pieces = [];
  for (const p of speechPieces(audio, RATE, 8)) {
    const last = pieces[pieces.length - 1];
    if (last && p.start - last.end < 0.7 * RATE && p.end - last.start <= 8 * RATE) last.end = p.end;
    else pieces.push({ ...p });
  }
  const result = await workerCall(worker, { type: 'asr', audio, pieces, size }, [], (d) => {
    if (d.type === 'download') onDownload(d);
    else if (d.type === 'asr') setProgress(0.2 + 0.6 * (d.done / d.total), `Listening… ${d.done} of ${d.total} stretches of speech${d.partial ? ` · ${d.partial} lines` : ''}`);
  });
  return result.segments;
}

async function deviceTranslate(worker, lines, setProgress) {
  const onDownload = downloadTracker(setProgress, 0.8, 0.88, 'Downloading the translation model…');
  setProgress(0.8, 'Loading the translation model…');
  const result = await workerCall(worker, { type: 'mt', lines }, [], (d) => {
    if (d.type === 'download' && d.stage === 'mt') onDownload(d);
    else if (d.type === 'mt') setProgress(0.88 + 0.12 * (d.done / d.total), `Translating… ${d.done} of ${d.total} lines`);
  });
  return result.lines;
}

async function chromeTranslate(translator, lines, setProgress, signal) {
  const out = [];
  for (let i = 0; i < lines.length; i++) {
    if (signal.aborted) throw new Cancelled();
    try { out.push(await translator.translate(lines[i])); } catch { out.push(''); }
    setProgress(0.8 + 0.2 * ((i + 1) / lines.length), `Translating with Chrome's built-in translator… ${i + 1} of ${lines.length}`);
  }
  return out;
}

// ───────────────────────────── own API key ─────────────────────────────

async function apiFetch(api, path, init, signal) {
  let res;
  try {
    res = await fetch(`${api.base}${path}`, { ...init, signal, headers: { Authorization: `Bearer ${api.key}`, ...(init.headers || {}) } });
  } catch (e) {
    if (signal.aborted) throw new Cancelled();
    throw new Error(`Could not reach ${api.name} from the browser. Check your connection; if it keeps failing, this provider may not accept requests sent directly from websites.`);
  }
  if (!res.ok) {
    let detail = '';
    try { const j = await res.json(); detail = j.error?.message || j.message || ''; } catch { /* not JSON */ }
    if (res.status === 401 || res.status === 403) throw new Error(`${api.name} rejected the API key${detail ? `: ${detail}` : '.'}`);
    if (res.status === 429) throw new Error(`${api.name} says the rate limit or quota was reached${detail ? `: ${detail}` : '.'}`);
    throw new Error(`${api.name} returned an error (HTTP ${res.status})${detail ? `: ${detail}` : '.'}`);
  }
  return res.json();
}

async function apiTranscribe(api, audio, setProgress, signal) {
  const segments = [];
  if (api.kind === 'whisper') {
    // At most 10 minutes per upload (about 19 MB as WAV), under the 25 MB limits.
    const blocks = splitAtQuiet(audio, RATE, 600, 8);
    for (let b = 0; b < blocks.length; b++) {
      setProgress(0.05 + 0.65 * (b / blocks.length), `Sending audio to ${api.name}… part ${b + 1} of ${blocks.length}`);
      const { start, end } = blocks[b];
      const form = new FormData();
      form.append('file', encodeWav(audio.subarray(start, end)), 'audio.wav');
      form.append('model', api.asr);
      form.append('language', 'tr');
      form.append('response_format', 'verbose_json');
      form.append('timestamp_granularities[]', 'segment');
      const json = await apiFetch(api, '/audio/transcriptions', { method: 'POST', body: form }, signal);
      const offset = start / RATE;
      for (const s of json.segments || []) segments.push({ start: offset + s.start, end: offset + s.end, text: s.text });
      if (!json.segments && json.text) segments.push({ start: offset, end: end / RATE, text: json.text });
    }
  } else {
    // Qwen returns text without timestamps, so each stretch of speech is sent on its own.
    const pieces = speechPieces(audio, RATE, 12);
    for (let i = 0; i < pieces.length; i++) {
      if (signal.aborted) throw new Cancelled();
      setProgress(0.05 + 0.65 * (i / pieces.length), `Sending audio to ${api.name}… ${i + 1} of ${pieces.length}`);
      const { start, end } = pieces[i];
      const dataUrl = await blobToDataUrl(encodeWav(audio.subarray(start, end)));
      const json = await apiFetch(api, '/chat/completions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: api.asr,
          messages: [{ role: 'user', content: [{ type: 'input_audio', input_audio: { data: dataUrl } }] }],
          asr_options: { language: 'tr', enable_itn: false },
          stream: false,
        }),
      }, signal);
      const text = json.choices?.[0]?.message?.content;
      segments.push({ start: start / RATE, end: end / RATE, text: typeof text === 'string' ? text : '' });
    }
  }
  return segments;
}

function blobToDataUrl(blob) {
  return new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(r.result);
    r.onerror = () => reject(r.error);
    r.readAsDataURL(blob);
  });
}

const SYSTEM_PROMPT = 'You translate Turkish video subtitles into natural, faithful English for language learners. '
  + 'You receive JSON {"lines": [...]} and reply with JSON {"lines": [...]} containing exactly one English line for each Turkish line, in the same order. '
  + 'Never merge, split, skip or explain lines.';

async function apiTranslate(api, lines, setProgress, signal) {
  const out = new Array(lines.length).fill('');
  const BATCH = 40;
  const ask = async (batch) => {
    const json = await apiFetch(api, '/chat/completions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: api.chat,
        temperature: 0.2,
        response_format: { type: 'json_object' },
        messages: [{ role: 'system', content: SYSTEM_PROMPT }, { role: 'user', content: JSON.stringify({ lines: batch }) }],
      }),
    }, signal);
    const text = json.choices?.[0]?.message?.content || '';
    try {
      const parsed = JSON.parse(text.replace(/^```(?:json)?\s*|\s*```$/g, ''));
      return Array.isArray(parsed.lines) ? parsed.lines.map(String) : null;
    } catch { return null; }
  };
  for (let i = 0; i < lines.length; i += BATCH) {
    const batch = lines.slice(i, i + BATCH);
    setProgress(0.72 + 0.28 * (i / lines.length), `Translating with ${api.name}… ${i} of ${lines.length} lines`);
    let got = await ask(batch);
    if (!got || got.length !== batch.length) {
      // The model did not keep the line count; fall back to one line at a time.
      got = [];
      for (const line of batch) got.push((await ask([line]))?.[0] || '');
    }
    got.forEach((t, k) => { out[i + k] = t; });
  }
  return out;
}
