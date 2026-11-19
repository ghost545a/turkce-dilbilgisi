// TRLearn web player: video by link or file, dual subtitles, click-to-look-up
// words, saved vocabulary with replay. Parsing and lookup run in Rust/WASM
// (src/player); this file handles the page.
import init, { PlayerDictionary, parse_dual_srt } from '../pkg/turkish_grammar_map.js';
import { setupAutoSubs } from './autosubs.js';
import { AudioVisual } from './visual.js';
import { Drawer, Modal, Materialise, trackHudHeight } from '../ui.js';

const $ = (id) => document.getElementById(id);
const VOCAB_KEY = 'trlearn.vocabulary';
const AUTOSAVE_KEY = 'trlearn.autosave';
const SAMPLE = { video: 'samples/sample_video.mp4', tr: 'samples/sample_tr.srt', en: 'samples/sample_en.srt' };

// ───────────────────────────── helpers ─────────────────────────────

function el(tag, className, text) {
  const n = document.createElement(tag);
  if (className) n.className = className;
  if (text !== undefined) n.textContent = text;
  return n;
}

function fmtTime(s) {
  const t = Math.max(0, Math.floor(s || 0));
  const h = Math.floor(t / 3600), m = Math.floor((t % 3600) / 60), sec = t % 60;
  const mm = String(m).padStart(h ? 2 : 2, '0'), ss = String(sec).padStart(2, '0');
  return h ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

function trLower(s) {
  return s.replace(/İ/g, 'i').replace(/I/g, 'ı').toLowerCase();
}

function cleanWord(raw) {
  return raw.replace(/^[^\p{L}]+|[^\p{L}]+$/gu, '');
}

function storageGet(key, fallback) {
  try { const v = localStorage.getItem(key); return v === null ? fallback : JSON.parse(v); } catch { return fallback; }
}
function storageSet(key, value) {
  try { localStorage.setItem(key, JSON.stringify(value)); return true; } catch { return false; }
}

let toastTimer = 0;
export function toast(text, action, onAction, ms = 5000) {
  const t = $('toast'), b = $('toast-action');
  $('toast-text').textContent = text;
  b.hidden = !action;
  b.textContent = action || '';
  b.onclick = action ? () => { t.hidden = true; onAction(); } : null;
  t.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => { t.hidden = true; }, ms);
}

function download(name, text, type = 'text/plain') {
  const url = URL.createObjectURL(new Blob([text], { type }));
  const a = el('a');
  a.href = url; a.download = name;
  document.body.append(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 2000);
}

async function fetchText(url, what) {
  let res;
  try { res = await fetch(url); } catch {
    throw new Error(`Could not download the ${what}. The site hosting it may not allow other websites to read it (CORS). Download the file and choose it from your device instead.`);
  }
  if (!res.ok) throw new Error(`Could not download the ${what} (HTTP ${res.status}).`);
  return res.text();
}

// ───────────────────────────── media ─────────────────────────────

export function youTubeId(link) {
  try {
    const u = new URL(link);
    const host = u.hostname.replace(/^www\.|^m\./, '');
    if (host === 'youtu.be') return u.pathname.slice(1).split('/')[0] || null;
    if (host === 'youtube.com' || host === 'music.youtube.com' || host === 'youtube-nocookie.com') {
      if (u.searchParams.get('v')) return u.searchParams.get('v');
      const m = u.pathname.match(/^\/(?:embed|shorts|live|v)\/([\w-]{6,})/);
      if (m) return m[1];
    }
  } catch { /* not a URL */ }
  return null;
}

class Html5Media {
  constructor(video) { this.v = video; this.kind = 'html5'; }
  play() { return this.v.play().catch(() => {}); }
  pause() { this.v.pause(); }
  paused() { return this.v.paused; }
  time() { return this.v.currentTime || 0; }
  duration() { return this.v.duration || 0; }
  seek(t) { this.v.currentTime = Math.max(0, Math.min(t, this.duration() || t)); }
  destroy() { this.v.pause(); this.v.removeAttribute('src'); this.v.load(); this.v.hidden = true; }
}

let ytApi = null;
function loadYouTubeApi() {
  if (!ytApi) {
    ytApi = new Promise((resolve, reject) => {
      if (window.YT && window.YT.Player) return resolve(window.YT);
      const prev = window.onYouTubeIframeAPIReady;
      window.onYouTubeIframeAPIReady = () => { prev && prev(); resolve(window.YT); };
      const s = document.createElement('script');
      s.src = 'https://www.youtube.com/iframe_api';
      s.onerror = () => { ytApi = null; reject(new Error('Could not load the YouTube player. Check your connection or ad blocker.')); };
      document.head.append(s);
    });
  }
  return ytApi;
}

class YouTubeMedia {
  constructor(player) { this.p = player; this.kind = 'youtube'; }
  play() { this.p.playVideo(); }
  pause() { this.p.pauseVideo(); }
  paused() { const s = this.p.getPlayerState?.(); return s !== 1 && s !== 3; }
  time() { return this.p.getCurrentTime?.() || 0; }
  duration() { return this.p.getDuration?.() || 0; }
  seek(t) { this.p.seekTo(Math.max(0, t), true); }
  destroy() { try { this.p.destroy(); } catch { /* already gone */ } $('yt-holder').hidden = true; $('yt-holder').innerHTML = '<div id="yt"></div>'; }
}

// ───────────────────────────── state ─────────────────────────────

const state = {
  media: null,        // Html5Media | YouTubeMedia
  source: null,       // { kind: 'url'|'file'|'youtube', url, file, name, key }
  subLinks: { tr: '', en: '' }, // links (for sharing), when subtitles came from links
  cues: [],
  active: -1,
  words: storageGet(VOCAB_KEY, []),
  dict: null,
  dictReady: null,
  replayStop: null,
  lastLookup: null,
};
if (!Array.isArray(state.words)) state.words = [];

const wordsDrawer = new Drawer($('words'), { side: 'right', button: $('btn-words'), initialFocus: () => $('words-close') });
const modals = { 'open-dialog': new Modal($('open-dialog')), 'auto-dialog': new Modal($('auto-dialog')) };
const lookupPop = new Materialise($('lookup'));
const narrowLayout = matchMedia('(max-width: 960px)');

const visual = new AudioVisual({
  live: $('viz'),
  note: $('viz-note'),
  strip: $('wave'),
  stripCanvas: $('wave-canvas'),
  tip: $('wave-tip'),
  getTime: () => (state.media ? state.media.time() : 0),
  isPaused: () => (state.media ? state.media.paused() : true),
  seek: (t) => state.media && state.media.seek(t),
});

function saveWords() {
  if (!storageSet(VOCAB_KEY, state.words)) toast('Could not save words in this browser (storage is blocked or full). Use Export to keep them.');
  renderWords();
}

// ───────────────────────────── lesson loading ─────────────────────────────

async function openMedia(source) {
  closeLookup();
  if (state.media) state.media.destroy();
  state.media = null;
  state.source = source;
  $('empty').hidden = true;
  const video = $('video');

  if (source.kind === 'youtube') {
    video.hidden = true;
    const YT = await loadYouTubeApi();
    $('yt-holder').hidden = false;
    await new Promise((resolve, reject) => {
      const player = new YT.Player('yt', {
        videoId: source.id,
        playerVars: { playsinline: 1, rel: 0, modestbranding: 1, cc_load_policy: 0, origin: location.origin },
        events: {
          onReady: () => { state.media = new YouTubeMedia(player); resolve(); },
          // YouTube's own captions would sit on top of the study subtitles.
          onStateChange: (e) => { if (e.data === 1) { try { player.unloadModule('captions'); } catch { /* no captions */ } } },
          onError: (e) => reject(new Error(`YouTube could not play this video (error ${e.data}). It may be private or not allowed in other websites.`)),
        },
      });
    });
  } else {
    $('yt-holder').hidden = true;
    video.hidden = false;
    video.removeAttribute('data-audio');
    if (!video.dataset.visualHooks) {
      video.dataset.visualHooks = '1';
      for (const ev of ['play', 'pause', 'seeked', 'seeking', 'timeupdate']) video.addEventListener(ev, () => visual.wake());
    }
    video.src = source.kind === 'file' ? URL.createObjectURL(source.file) : source.url;
    state.media = new Html5Media(video);
    await new Promise((resolve, reject) => {
      const ok = () => { cleanup(); video.dataset.audio = video.videoWidth ? '0' : '1'; resolve(); };
      const bad = () => {
        cleanup();
        const insecure = source.kind === 'url' && location.protocol === 'https:' && source.url.startsWith('http:');
        reject(new Error(insecure
          ? 'This link starts with http://, which browsers block on a secure (https://) page. Use an https:// link or download the file.'
          : 'The browser could not play this file or link. It may not be a direct media file, or its format is not supported.'));
      };
      const cleanup = () => { video.removeEventListener('loadedmetadata', ok); video.removeEventListener('error', bad); };
      video.addEventListener('loadedmetadata', ok);
      video.addEventListener('error', bad);
    });
  }
  $('lesson-name').textContent = source.name;
  // Audio files (no picture) get the live visual and the waveform strip.
  if (source.kind !== 'youtube' && video.dataset.audio === '1') visual.load(source);
  else visual.clear();
  document.dispatchEvent(new CustomEvent('lesson-media', { detail: source }));
}

export function setCues(cues, warnings = []) {
  state.cues = cues;
  state.active = -1;
  visual.setCues(cues);
  visual.setActive(-1);
  renderTranscript();
  renderStage(-1);
  if (warnings.length) toast(warnings.join(' '), null, null, 8000);
}

function parseAndSet(trText, enText) {
  const res = JSON.parse(parse_dual_srt(trText, enText || ''));
  if (!res.ok) throw new Error(res.error);
  setCues(res.cues, res.warnings);
}

async function readSub(file, link, label) {
  if (file) return file.text();
  if (link) return fetchText(link, `${label} subtitles`);
  return '';
}

async function openLesson({ videoLink, videoFile, trLink, trFile, enLink, enFile }) {
  let source;
  if (videoFile) {
    source = { kind: 'file', file: videoFile, name: videoFile.name, key: videoFile.name };
  } else if (videoLink) {
    const id = youTubeId(videoLink);
    let url;
    try { url = new URL(videoLink, location.href).href; } catch { throw new Error('That video link is not a valid web address.'); }
    source = id
      ? { kind: 'youtube', id, url: `https://www.youtube.com/watch?v=${id}`, name: `YouTube · ${id}`, key: `youtube:${id}` }
      : { kind: 'url', url, name: decodeURIComponent(url.split('/').pop().split('?')[0]) || url, key: url };
  } else {
    throw new Error('Paste a video link or choose a file.');
  }

  // Read subtitles before touching the player, so a bad file leaves the old lesson intact.
  const [trText, enText] = await Promise.all([readSub(trFile, trLink, 'Turkish'), readSub(enFile, enLink, 'English')]);
  if (!trText && enText) throw new Error('Add the Turkish subtitles too (the English file alone cannot be studied).');
  let parsed = null;
  if (trText) {
    parsed = JSON.parse(parse_dual_srt(trText, enText));
    if (!parsed.ok) throw new Error(parsed.error);
  }

  try {
    await openMedia(source);
  } catch (e) {
    resetPlayer();
    throw e;
  }
  state.subLinks = { tr: trFile ? '' : (trLink || ''), en: enFile ? '' : (enLink || '') };
  if (parsed) setCues(parsed.cues, parsed.warnings);
  else {
    setCues([]);
    toast(source.kind === 'youtube'
      ? 'No subtitles yet. Add Turkish and English .srt files with Open.'
      : 'No subtitles yet. Add .srt files with Open, or create them with Auto subtitles.', null, null, 7000);
  }
  updateButtons();
}

/** Back to the empty start screen (after a video that could not be opened). */
function resetPlayer() {
  visual.clear();
  if (state.media) state.media.destroy();
  state.media = null;
  state.source = null;
  state.subLinks = { tr: '', en: '' };
  $('video').hidden = true;
  $('yt-holder').hidden = true;
  $('empty').hidden = false;
  $('lesson-name').textContent = '';
  setCues([]);
  updateButtons();
  document.dispatchEvent(new CustomEvent('lesson-media', { detail: null }));
}

function updateButtons() {
  const src = state.source;
  $('btn-share').disabled = !src || src.kind === 'file';
  $('btn-share').title = src && src.kind === 'file'
    ? 'A file from your device cannot be shared as a link'
    : 'Copy a link to this lesson';
}

// ───────────────────────────── subtitles & transcript ─────────────────────────────

function cueAt(t) {
  const cues = state.cues;
  let lo = 0, hi = cues.length - 1, found = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (cues[mid].start <= t) { found = mid; lo = mid + 1; } else hi = mid - 1;
  }
  return found >= 0 && t <= cues[found].end + 0.05 ? found : -1;
}

function savedSet() {
  return new Set(state.words.map((w) => trLower(w.target_word)));
}

function renderStage(i) {
  const tr = $('sub-tr'), en = $('sub-en');
  tr.replaceChildren();
  en.textContent = '';
  if (i < 0) return;
  const cue = state.cues[i];
  const saved = savedSet();
  cue.tr.split(/\s+/).filter(Boolean).forEach((token, k) => {
    if (k) tr.append(' ');
    const word = cleanWord(token);
    if (!word) { tr.append(token); return; }
    const b = el('button', 'word', token);
    b.type = 'button';
    if (saved.has(trLower(word))) b.classList.add('saved');
    b.addEventListener('click', () => lookUp(token, cue, b));
    tr.append(b);
  });
  en.textContent = cue.en;
}

function renderTranscript() {
  const list = $('transcript');
  list.replaceChildren(...state.cues.map((cue, i) => {
    const li = el('li');
    const b = el('button');
    b.type = 'button';
    const tr = el('span', 't-tr', cue.tr); tr.lang = 'tr';
    b.append(el('span', 't-time', fmtTime(cue.start)), tr, el('span', 't-en', cue.en));
    b.addEventListener('click', () => { if (state.media) { state.media.seek(cue.start + 0.01); state.media.play(); } });
    li.append(b);
    li.dataset.i = i;
    return li;
  }));
  $('transcript-empty').hidden = state.cues.length > 0;
}

function setActive(i) {
  if (i === state.active) return;
  const list = $('transcript');
  list.children[state.active]?.classList.remove('active');
  state.active = i;
  visual.setActive(i);
  renderStage(i);
  const li = list.children[i];
  if (li) {
    li.classList.add('active');
    if (!list.matches(':hover')) li.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }
}

function tick() {
  if (state.media && state.cues.length) setActive(cueAt(state.media.time()));
  if (state.replayStop && state.media && state.media.time() >= state.replayStop) {
    state.replayStop = null;
    state.media.pause();
  }
  // The word card belongs to the paused moment: once playback resumes (Space,
  // the play button, YouTube's controls…), it goes away.
  if (state.media && !state.media.paused() && !$('lookup').hidden && $('lookup').dataset.closing !== '1') {
    closeLookup();
  }
}

function step(dir) {
  if (!state.media || !state.cues.length) return;
  const t = state.media.time();
  let i;
  if (dir > 0) i = state.cues.findIndex((c) => c.start > t + 0.05);
  else {
    const cur = cueAt(t);
    i = cur >= 0 && t - state.cues[cur].start > 1 ? cur : Math.max(0, (cur >= 0 ? cur : state.cues.findLastIndex((c) => c.start < t)) - 1);
  }
  if (i >= 0 && i < state.cues.length) state.media.seek(state.cues[i].start + 0.01);
}

// ───────────────────────────── dictionary lookup ─────────────────────────────

function loadDictionary() {
  if (!state.dictReady) {
    state.dictReady = fetch('tr_en_dictionary.tsv')
      .then((r) => { if (!r.ok) throw new Error(`HTTP ${r.status}`); return r.text(); })
      .then((tsv) => { state.dict = new PlayerDictionary(tsv); return state.dict; })
      .catch((e) => { state.dictReady = null; throw e; });
  }
  return state.dictReady;
}

async function lookUp(token, cue, button) {
  const word = cleanWord(token);
  if (!word) return;
  state.media?.pause();
  document.querySelectorAll('.word.active').forEach((w) => w.classList.remove('active'));
  button?.classList.add('active');
  $('lookup-word').textContent = word;
  $('lookup-meaning').textContent = 'Looking up…';
  $('lookup-meaning').classList.remove('missing');
  $('lookup-note').textContent = '';
  $('lookup-save').hidden = true;
  showLookupCard(button);

  let found = null;
  try {
    const dict = state.dict || await loadDictionary();
    found = JSON.parse(dict.lookup(word));
  } catch (e) {
    $('lookup-meaning').textContent = `The dictionary could not be loaded (${e.message}).`;
    $('lookup-meaning').classList.add('missing');
    return;
  }
  if (!found) {
    $('lookup-meaning').textContent = 'Not in the offline dictionary.';
    $('lookup-meaning').classList.add('missing');
    state.lastLookup = null;
    return;
  }
  $('lookup-meaning').textContent = found.translation;
  $('lookup-note').textContent = found.note;

  const entry = {
    id: `${trLower(word)}-${Math.round(cue.start * 1000)}`,
    target_word: word,
    english_translation: found.translation,
    target_sentence: cue.tr,
    english_sentence: cue.en,
    timestamp: cue.start,
    word_note: found.note,
    source_media: state.source?.key || '',
  };
  state.lastLookup = { entry, button };
  // Saving is the learner's choice; the desktop app's save-on-click is a setting.
  if (autoSave() && savedIndex(entry) < 0) storeWord(entry);
  updateSaveButton();
}

function savedIndex(entry) {
  return state.words.findIndex((w) => trLower(w.target_word) === trLower(entry.target_word) && w.target_sentence === entry.target_sentence);
}

function storeWord(entry) {
  const i = savedIndex(entry);
  if (i >= 0) state.words[i] = entry; else state.words.push(entry);
  saveWords();
  renderStage(state.active);
}

function updateSaveButton() {
  const save = $('lookup-save');
  const last = state.lastLookup;
  save.hidden = !last;
  if (!last) return;
  const saved = savedIndex(last.entry) >= 0;
  save.textContent = saved ? 'Saved ✓' : 'Save word';
  save.classList.toggle('primary', !saved);
  save.setAttribute('aria-pressed', String(saved));
  save.title = saved ? 'Click to remove it from your saved words' : 'Add this word and its sentence to your saved words';
}

function autoSave() {
  try { return localStorage.getItem(AUTOSAVE_KEY) === '1'; } catch { return false; }
}

/** Shows the word card growing out of the clicked word. */
function showLookupCard(button) {
  const card = $('lookup');
  card.hidden = false;
  if (narrowLayout.matches || !button) {
    card.style.left = '';
    card.style.top = '';
    lookupPop.show('50% 100%');
    return;
  }
  const stage = card.offsetParent.getBoundingClientRect();
  const b = button.getBoundingClientRect();
  const w = card.offsetWidth;
  const centre = b.left + b.width / 2 - stage.left;
  const left = Math.max(0, Math.min(stage.width - w, centre - w / 2));
  card.style.left = `${left}px`;
  card.style.top = `${b.bottom - stage.top + 10}px`;
  card.style.setProperty('--arrow-x', `${centre - left}px`);
  lookupPop.show(`${centre - left}px 0px`);
}

function closeLookup() {
  if (!$('lookup').hidden) lookupPop.hide();
  document.querySelectorAll('.word.active').forEach((w) => w.classList.remove('active'));
}

// ───────────────────────────── saved words ─────────────────────────────

function removeWord(id) {
  const index = state.words.findIndex((w) => w.id === id);
  if (index < 0) return;
  const [word] = state.words.splice(index, 1);
  saveWords();
  renderStage(state.active);
  toast(`Removed “${word.target_word}”.`, 'Undo', () => {
    state.words.splice(Math.min(index, state.words.length), 0, word);
    saveWords();
    renderStage(state.active);
  }, 6000);
}

function replay(word) {
  if (!state.media) {
    toast(`Open ${word.source_media || 'the video'} to replay “${word.target_word}”.`);
    return;
  }
  if (word.source_media && state.source && word.source_media !== state.source.key) {
    toast(`“${word.target_word}” was saved from ${word.source_media}. Open that video to replay it.`);
    return;
  }
  const start = Math.max(0, word.timestamp - 0.35);
  state.media.seek(start);
  state.replayStop = start + 5;
  state.media.play();
}

function highlightIn(sentence, word) {
  const frag = document.createDocumentFragment();
  const lower = trLower(sentence), w = trLower(word);
  const at = w ? lower.indexOf(w) : -1;
  if (at < 0) { frag.append(sentence); return frag; }
  frag.append(sentence.slice(0, at), el('mark', '', sentence.slice(at, at + word.length)), sentence.slice(at + word.length));
  return frag;
}

function renderWords() {
  $('words-count').textContent = state.words.length;
  const list = $('words-list');
  const items = [...state.words].reverse().map((w) => {
    const li = el('li');
    const top = el('div', 'w-top');
    const word = el('span', 'w-word', w.target_word); word.lang = 'tr';
    top.append(word, el('span', 'w-time', fmtTime(w.timestamp)));
    li.append(top, el('div', 'w-meaning', w.english_translation || '—'));
    if (w.word_note) li.append(el('div', 'w-note', w.word_note));
    if (w.target_sentence) {
      const s = el('div', 'w-sentence'); s.lang = 'tr';
      s.append(highlightIn(w.target_sentence, w.target_word));
      li.append(s);
    }
    if (w.english_sentence) li.append(el('div', 'w-en', w.english_sentence));
    if (w.source_media) li.append(el('div', 'w-source', w.source_media));
    const actions = el('div', 'w-actions');
    const rp = el('button', '', 'Replay'); rp.type = 'button'; rp.addEventListener('click', () => replay(w));
    const rm = el('button', '', 'Remove'); rm.type = 'button'; rm.addEventListener('click', () => removeWord(w.id));
    actions.append(rp, rm);
    li.append(actions);
    return li;
  });
  list.replaceChildren(...items);
  $('words-empty').hidden = state.words.length > 0;
}

function setWordsOpen(open) {
  if (open) wordsDrawer.open(); else wordsDrawer.close();
}

function importWords(text) {
  let list;
  try { list = JSON.parse(text); } catch { throw new Error('That file is not valid JSON.'); }
  if (!Array.isArray(list)) throw new Error('Expected a list of words (vocabulary.json).');
  let added = 0;
  for (const w of list) {
    if (!w || typeof w.target_word !== 'string') continue;
    const entry = {
      id: String(w.id || `${trLower(w.target_word)}-${Date.now()}-${added}`),
      target_word: w.target_word,
      english_translation: String(w.english_translation || ''),
      target_sentence: String(w.target_sentence || ''),
      english_sentence: String(w.english_sentence || ''),
      timestamp: Number(w.timestamp) || 0,
      word_note: String(w.word_note || ''),
      source_media: String(w.source_media || ''),
    };
    const i = state.words.findIndex((x) => trLower(x.target_word) === trLower(entry.target_word) && x.target_sentence === entry.target_sentence);
    if (i >= 0) state.words[i] = entry; else { state.words.push(entry); added++; }
  }
  saveWords();
  return added;
}

// ───────────────────────────── share links ─────────────────────────────

function shareUrl() {
  const src = state.source;
  if (!src || src.kind === 'file') return null;
  const u = new URL(location.href.split('?')[0].split('#')[0]);
  const isSample = src.url && src.url === new URL(SAMPLE.video, location.href).href;
  if (isSample) { u.searchParams.set('sample', '1'); return u.href; }
  u.searchParams.set('v', src.url);
  if (state.subLinks.tr) u.searchParams.set('tr', state.subLinks.tr);
  if (state.subLinks.en) u.searchParams.set('en', state.subLinks.en);
  return u.href;
}

// ───────────────────────────── open dialog ─────────────────────────────

function openModal(id) { modals[id].open(); }
function closeModal(id) { modals[id].close(); }

function bindFileName(inputId, labelId) {
  $(inputId).addEventListener('change', () => { $(labelId).textContent = $(inputId).files[0]?.name || ''; });
}

function bindOpenDialog() {
  ['in-video-file', 'in-tr-file', 'in-en-file'].forEach((id) => bindFileName(id, `${id}-name`));
  $('open-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const err = $('open-error');
    err.textContent = '';
    const submit = e.submitter || $('open-form').querySelector('button[type=submit]');
    submit.disabled = true;
    submit.textContent = 'Opening…';
    try {
      await openLesson({
        videoLink: $('in-video').value.trim(),
        videoFile: $('in-video-file').files[0],
        trLink: $('in-tr').value.trim(),
        trFile: $('in-tr-file').files[0],
        enLink: $('in-en').value.trim(),
        enFile: $('in-en-file').files[0],
      });
      closeModal('open-dialog');
    } catch (ex) {
      err.textContent = ex.message;
    } finally {
      submit.disabled = false;
      submit.textContent = 'Open';
    }
  });
  // A link and a file for the same slot: the most recent choice wins.
  [['in-video', 'in-video-file'], ['in-tr', 'in-tr-file'], ['in-en', 'in-en-file']].forEach(([link, file]) => {
    $(link).addEventListener('input', () => { if ($(link).value) { $(file).value = ''; $(`${file}-name`).textContent = ''; } });
    $(file).addEventListener('change', () => { if ($(file).files[0]) $(link).value = ''; });
  });
}

// ───────────────────────────── keyboard & buttons ─────────────────────────────

function bind() {
  $('btn-open').addEventListener('click', () => openModal('open-dialog'));
  $('btn-open-2').addEventListener('click', () => openModal('open-dialog'));
  $('btn-sample').addEventListener('click', () => openSample());
  $('btn-words').addEventListener('click', () => setWordsOpen(!wordsDrawer.isOpen));
  $('autosave').checked = autoSave();
  $('autosave').addEventListener('change', (e) => {
    try { localStorage.setItem(AUTOSAVE_KEY, e.target.checked ? '1' : '0'); } catch { /* storage blocked */ }
  });
  $('words-close').addEventListener('click', () => setWordsOpen(false));
  $('lookup-close').addEventListener('click', closeLookup);
  $('lookup-resume').addEventListener('click', () => { closeLookup(); state.media?.play(); });
  $('lookup-save').addEventListener('click', () => {
    const last = state.lastLookup;
    if (!last) return;
    const i = savedIndex(last.entry);
    if (i >= 0) removeWord(state.words[i].id);
    else { storeWord(last.entry); toast(`Saved “${last.entry.target_word}”.`); }
    updateSaveButton();
  });
  $('btn-share').addEventListener('click', async () => {
    const url = shareUrl();
    if (!url) return;
    const local = !state.subLinks.tr && state.cues.length && !url.includes('sample=1');
    try { await navigator.clipboard.writeText(url); toast(local ? 'Link copied. Subtitles chosen from your device are not part of the link.' : 'Link to this lesson copied.'); }
    catch { window.prompt('Copy this link:', url); }
  });
  $('words-export').addEventListener('click', () => {
    if (!state.words.length) { toast('No saved words to export yet.'); return; }
    download('vocabulary.json', JSON.stringify(state.words, null, 2), 'application/json');
  });
  $('words-import').addEventListener('change', async (e) => {
    const f = e.target.files[0];
    e.target.value = '';
    if (!f) return;
    try { const n = importWords(await f.text()); toast(`Imported ${n} new word${n === 1 ? '' : 's'}.`); }
    catch (ex) { toast(ex.message); }
  });
  document.querySelectorAll('.modal [data-close]').forEach((b) => b.addEventListener('click', () => closeModal(b.closest('.modal').id)));
  document.querySelectorAll('.modal').forEach((m) => m.addEventListener('mousedown', (e) => { if (e.target === m) closeModal(m.id); }));

  window.addEventListener('keydown', (e) => {
    const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement;
    if (e.key === 'Escape') {
      const modal = Object.entries(modals).find(([, m]) => m.isOpen);
      if (modal) return closeModal(modal[0]);
      if (!$('lookup').hidden) return closeLookup();
      if (wordsDrawer.isOpen) return setWordsOpen(false);
      return;
    }
    if (typing || e.ctrlKey || e.metaKey || e.altKey) return;
    if ([...document.querySelectorAll('.modal')].some((m) => !m.hidden)) return;
    const m = state.media;
    switch (e.key) {
      case ' ':
        if (e.target instanceof HTMLButtonElement && e.target.closest('.subs, .transcript, .lookup, .words')) return;
        if (!m) return;
        e.preventDefault();
        if (e.target === $('video')) return; // the native control already toggles
        m.paused() ? m.play() : m.pause();
        break;
      case 'ArrowLeft': if (m) { e.preventDefault(); m.seek(m.time() - 5); } break;
      case 'ArrowRight': if (m) { e.preventDefault(); m.seek(m.time() + 5); } break;
      case 'ArrowUp': e.preventDefault(); step(-1); break;
      case 'ArrowDown': e.preventDefault(); step(1); break;
      case 'o': case 'O': openModal('open-dialog'); break;
      case 'w': case 'W': setWordsOpen(!wordsDrawer.isOpen); break;
      default: return;
    }
  });
}

async function openSample() {
  try {
    await openLesson({ videoLink: SAMPLE.video, trLink: SAMPLE.tr, enLink: SAMPLE.en });
  } catch (e) { toast(e.message); }
}

async function openFromQuery() {
  const q = new URLSearchParams(location.search);
  if (q.get('sample')) return openSample();
  const v = q.get('v');
  if (!v) return;
  try {
    await openLesson({ videoLink: v, trLink: q.get('tr') || '', enLink: q.get('en') || '' });
  } catch (e) {
    $('in-video').value = v;
    $('in-tr').value = q.get('tr') || '';
    $('in-en').value = q.get('en') || '';
    openModal('open-dialog');
    $('open-error').textContent = e.message;
  }
}

// ───────────────────────────── start ─────────────────────────────

async function main() {
  const status = $('status');
  try {
    await init();
  } catch (e) {
    status.classList.remove('hidden');
    status.classList.add('error');
    status.textContent = 'Could not start the player engine.\n\nServe this folder over http:// (not file://).\n\n' + e;
    return;
  }
  bind();
  bindOpenDialog();
  trackHudHeight();
  renderWords();
  updateButtons();
  setupAutoSubs({
    getSource: () => state.source,
    getMedia: () => state.media,
    setCues,
    toast,
    download,
    openModal,
    onSubLinksCleared: () => { state.subLinks = { tr: '', en: '' }; },
  });
  // A timer rather than requestAnimationFrame, so subtitles keep up even
  // when the browser stops painting (background tab, covered window).
  setInterval(tick, 100);
  loadDictionary().catch(() => { /* reported when a word is clicked */ });
  openFromQuery();
  window.trlearn = { state, openLesson, lookUp }; // console handle for debugging
}

main();
