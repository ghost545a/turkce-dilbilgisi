// On-device speech recognition (Whisper) and translation (opus-mt-tr-en),
// run with Transformers.js in a worker so the page stays responsive.
import { pipeline, env } from 'https://cdn.jsdelivr.net/npm/@huggingface/transformers@3.8.1';
import { RATE, splitAtQuiet } from './audio-utils.js';

env.allowLocalModels = false;

const BLOCK_SEC = 120;
let asr = null, asrKey = '';
let mt = null;

const post = (msg) => self.postMessage(msg);

function progressFor(stage) {
  return (p) => {
    if (p.status === 'progress' && p.total) {
      post({ type: 'download', stage, file: p.file, loaded: p.loaded, total: p.total });
    } else if (p.status === 'done') {
      post({ type: 'download', stage, file: p.file, done: true });
    }
  };
}

const MODELS = {
  tiny: 'onnx-community/whisper-tiny',
  base: 'onnx-community/whisper-base',
  small: 'onnx-community/whisper-small',
  turbo: 'onnx-community/whisper-large-v3-turbo',
};

async function loadAsr(size, webgpu, f16) {
  const key = `${size}:${webgpu}:${f16}`;
  if (asr && asrKey === key) return asr;
  asr = null;
  let options;
  if (size === 'turbo') {
    // Large model: 4-bit weights keep the download near 540 MB (720 MB without 16-bit GPU floats).
    if (!webgpu) throw new Error('The Turbo model needs a graphics card (WebGPU). Choose Base or Small instead.');
    options = { device: 'webgpu', dtype: f16 ? { encoder_model: 'q4f16', decoder_model_merged: 'q4f16' } : { encoder_model: 'q4', decoder_model_merged: 'q4' } };
  } else {
    options = webgpu
      ? { device: 'webgpu', dtype: { encoder_model: 'fp32', decoder_model_merged: 'q4' } }
      : { device: 'wasm', dtype: 'q8' };
  }
  asr = await pipeline('automatic-speech-recognition', MODELS[size], {
    ...options,
    progress_callback: progressFor('asr'),
  });
  asrKey = key;
  return asr;
}

async function transcribe({ audio, size, webgpu, f16 }) {
  let model;
  try {
    model = await loadAsr(size, webgpu, f16);
  } catch (e) {
    if (!webgpu || size === 'turbo') throw e;
    post({ type: 'note', text: 'The graphics card could not run the model; using the CPU instead (slower).' });
    model = await loadAsr(size, false, false);
  }
  const blocks = splitAtQuiet(audio, RATE, BLOCK_SEC);
  const segments = [];
  post({ type: 'asr', done: 0, total: blocks.length });
  for (let b = 0; b < blocks.length; b++) {
    const { start, end } = blocks[b];
    const offset = start / RATE;
    const out = await model(audio.subarray(start, end), {
      language: 'turkish',
      task: 'transcribe',
      return_timestamps: true,
      chunk_length_s: 30,
      stride_length_s: 5,
    });
    for (const c of out.chunks || []) {
      const [s, e] = c.timestamp || [];
      segments.push({
        start: offset + (Number.isFinite(s) ? s : 0),
        end: Number.isFinite(e) ? offset + e : end / RATE,
        text: c.text,
      });
    }
    post({ type: 'asr', done: b + 1, total: blocks.length, partial: segments.length });
  }
  return segments;
}

async function translate({ lines }) {
  if (!mt) {
    mt = await pipeline('translation', 'Xenova/opus-mt-tr-en', { dtype: 'q8', progress_callback: progressFor('mt') });
  }
  const out = [];
  const BATCH = 8;
  post({ type: 'mt', done: 0, total: lines.length });
  for (let i = 0; i < lines.length; i += BATCH) {
    const batch = lines.slice(i, i + BATCH);
    const res = await mt(batch, { max_new_tokens: 256 });
    for (const r of Array.isArray(res) ? res : [res]) out.push(r.translation_text || '');
    post({ type: 'mt', done: Math.min(lines.length, i + BATCH), total: lines.length });
  }
  return out;
}

self.onmessage = async ({ data }) => {
  try {
    if (data.type === 'asr') post({ type: 'asr-result', segments: await transcribe(data) });
    else if (data.type === 'mt') post({ type: 'mt-result', lines: await translate(data) });
  } catch (e) {
    post({ type: 'error', message: e && e.message ? e.message : String(e) });
  }
};
