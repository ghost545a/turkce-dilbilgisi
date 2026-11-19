// On-device Turkish speech recognition with the seda models (atasoglu,
// CC BY-SA 4.0) running in sherpa-onnx's WebAssembly build (Apache-2.0).
// A classic worker, because the sherpa-onnx build expects a global `Module`.
//
// Messages in:  {type: 'asr', audio: Float32Array (16 kHz mono), pieces: [{start, end}] in samples, size: 'tiny' | 'v0.2'}
// Messages out: {type: 'download', stage: 'asr', file, loaded, total} · {type: 'asr', done, total}
//               {type: 'asr-result', segments: [{start, end, text}]} · {type: 'error', message}

const VENDOR = new URL('vendor/sherpa-onnx/', self.location.href).href;
const REPOS = { tiny: 'atasoglu/seda-v0.2-tiny', 'v0.2': 'atasoglu/seda-v0.2' };
// The `final/` export decodes a whole utterance with full context (the
// model card's "accurate" second pass), which suits finished recordings.
const FILES = ['final/encoder.int8.onnx', 'final/decoder.onnx', 'final/joiner.int8.onnx', 'tokens.txt'];
const RATE = 16000;

const post = (m) => self.postMessage(m);
let runtime = null;
let recognizer = null, recognizerKey = '';

function loadRuntime() {
  if (!runtime) {
    runtime = new Promise((resolve, reject) => {
      self.Module = {
        locateFile: (path) => VENDOR + path,
        // The release bundles an English model package; hand it an empty one
        // so it is never downloaded, and write the Turkish model files instead.
        getPreloadedPackage: () => new ArrayBuffer(0),
        onRuntimeInitialized: () => resolve(self.Module),
        onAbort: (what) => reject(new Error(`The speech engine could not start (${what}).`)),
        print: () => {},
        printErr: () => {},
      };
      try {
        importScripts(VENDOR + 'sherpa-onnx-wasm-main-asr.js', VENDOR + 'sherpa-onnx-asr.js');
      } catch (e) {
        reject(e);
      }
    });
  }
  return runtime;
}

/** Downloads a model file (kept in the browser's cache after the first time). */
async function fetchModelFile(repo, path) {
  const url = `https://huggingface.co/${repo}/resolve/main/${path}`;
  let cache = null;
  try { cache = await caches.open('seda-models'); } catch { /* no Cache API */ }
  const cached = cache && await cache.match(url);
  if (cached) {
    const buf = await cached.arrayBuffer();
    post({ type: 'download', stage: 'asr', file: path, loaded: buf.byteLength, total: buf.byteLength, done: true });
    return new Uint8Array(buf);
  }
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Could not download ${path} (HTTP ${res.status}).`);
  const total = Number(res.headers.get('content-length')) || 0;
  const reader = res.body.getReader();
  const chunks = [];
  let loaded = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    loaded += value.length;
    if (total) post({ type: 'download', stage: 'asr', file: path, loaded, total });
  }
  const out = new Uint8Array(loaded);
  let o = 0;
  for (const c of chunks) { out.set(c, o); o += c.length; }
  post({ type: 'download', stage: 'asr', file: path, done: true });
  if (cache) { try { await cache.put(url, new Response(out.slice())); } catch { /* quota */ } }
  return out;
}

async function loadRecognizer(size) {
  if (recognizer && recognizerKey === size) return recognizer;
  const repo = REPOS[size];
  if (!repo) throw new Error(`Unknown seda model "${size}".`);
  const [Module, ...files] = await Promise.all([loadRuntime(), ...FILES.map((f) => fetchModelFile(repo, f))]);
  const names = ['encoder.onnx', 'decoder.onnx', 'joiner.onnx', 'tokens.txt'];
  names.forEach((name, i) => {
    try { Module.FS_unlink('/' + name); } catch { /* not there yet */ }
    Module.FS_createDataFile('/', name, files[i], true, true, true);
  });
  if (recognizer) { try { recognizer.free(); } catch { /* already freed */ } }
  recognizer = self.createOnlineRecognizer(Module, {
    featConfig: { sampleRate: RATE, featureDim: 80 },
    modelConfig: {
      transducer: { encoder: '/encoder.onnx', decoder: '/decoder.onnx', joiner: '/joiner.onnx' },
      paraformer: { encoder: '', decoder: '' },
      zipformer2Ctc: { model: '' },
      nemoCtc: { model: '' },
      toneCtc: { model: '' },
      tokens: '/tokens.txt',
      numThreads: 1,
      provider: 'cpu',
      debug: 0,
      modelType: '',
      modelingUnit: '',
      bpeVocab: '',
    },
    decodingMethod: 'modified_beam_search',
    maxActivePaths: 4,
    enableEndpoint: 0,
    rule1MinTrailingSilence: 2.4,
    rule2MinTrailingSilence: 1.2,
    rule3MinUtteranceLength: 20,
    hotwordsFile: '',
    hotwordsScore: 1.5,
    ctcFstDecoderConfig: { graph: '', maxActive: 3000 },
    ruleFsts: '',
    ruleFars: '',
  });
  recognizerKey = size;
  return recognizer;
}

function decodePiece(rec, samples) {
  const stream = rec.createStream();
  try {
    stream.acceptWaveform(RATE, samples);
    // The full-context model works in 5.12 s blocks: pad with enough
    // silence that the last block of speech is decoded too.
    stream.acceptWaveform(RATE, new Float32Array(Math.round(RATE * 6)));
    stream.inputFinished();
    while (rec.isReady(stream)) rec.decode(stream);
    return (rec.getResult(stream).text || '').trim();
  } finally {
    stream.free();
  }
}

async function transcribe({ audio, pieces, size }) {
  const rec = await loadRecognizer(size);
  const segments = [];
  post({ type: 'asr', done: 0, total: pieces.length });
  for (let i = 0; i < pieces.length; i++) {
    const { start, end } = pieces[i];
    const text = decodePiece(rec, audio.subarray(start, end));
    if (text) {
      // Spoken-form lowercase output: capitalise the start of each line.
      segments.push({ start: start / RATE, end: end / RATE, text: text.charAt(0).toLocaleUpperCase('tr') + text.slice(1) });
    }
    if (i % 4 === 3 || i === pieces.length - 1) post({ type: 'asr', done: i + 1, total: pieces.length, partial: segments.length });
  }
  return segments;
}

self.onmessage = async ({ data }) => {
  try {
    if (data.type === 'asr') post({ type: 'asr-result', segments: await transcribe(data) });
  } catch (e) {
    post({ type: 'error', message: e && e.message ? e.message : String(e) });
  }
};
