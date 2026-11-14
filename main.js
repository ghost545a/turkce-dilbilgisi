// Thin bootstrapper: loads the WASM module, forwards DOM events to it and
// renders the Rule Inspector drawer from the JSON the Rust side hands back.
import init, { App, dict_search } from './pkg/turkish_grammar_map.js';

const $ = (id) => document.getElementById(id);
const canvas = $('map');
const statusEl = $('status');
const drawer = $('inspector');
const drawerBody = $('inspector-body');
const searchInput = $('search');
const searchCount = $('search-count');

const dict = $('dictionary');
const dictInput = $('dict-input');
const dictCount = $('dict-count');
const dictList = $('dict-list');
const dictButton = $('btn-dict');

let app = null;
let dictKind = 'verbs';

// Which grammar rule explains each case a verb can govern.
const CASE_RULES = {
  '-(y)A': 'Yönelme Hâli -(y)A',
  '-DA': 'Bulunma Hâli -DA',
  '-DAn': 'Ayrılma Hâli -DAn',
  '-(y)I': 'Belirtme Hâli -(y)I',
  '-(y)lA': 'Vasıta Hâli -(y)lA',
};

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

/** "Ev[ler] güzel." → text nodes with the bracketed part wrapped in <mark>. */
function highlighted(sentence) {
  const frag = document.createDocumentFragment();
  for (const part of sentence.split(/(\[[^\]]*\])/)) {
    if (part.startsWith('[') && part.endsWith(']')) {
      frag.append(el('mark', '', part.slice(1, -1)));
    } else if (part) {
      frag.append(part);
    }
  }
  return frag;
}

function section(title, ...children) {
  const s = el('section', 'section');
  s.append(el('h3', '', title), ...children);
  return s;
}

function chip(node) {
  const c = el('button', node.tr.startsWith('-') ? 'chip mono' : 'chip', node.tr);
  c.type = 'button';
  c.append(el('small', '', node.en));
  c.addEventListener('click', () => app.focus(node.id));
  return c;
}

function openInspector(json) {
  const rule = JSON.parse(json);
  if (!rule) return;
  drawerBody.replaceChildren();

  if (rule.path.length) {
    const crumbs = el('nav', 'crumbs');
    rule.path.forEach((p, i) => {
      if (i) crumbs.append(el('span', '', '›'));
      const b = el('button', '', p.tr);
      b.type = 'button';
      b.addEventListener('click', () => app.focus(p.id));
      crumbs.append(b);
    });
    drawerBody.append(crumbs);
  }

  const row = el('div', 'title-row');
  const titles = el('div');
  titles.append(
    el('h2', rule.tr.startsWith('-') ? 'rule-tr mono' : 'rule-tr', rule.tr),
    el('p', 'rule-en', rule.en),
  );
  const badge = el('span', 'cefr', rule.level);
  badge.dataset.band = rule.level[0];
  badge.title = 'CEFR level';
  row.append(titles, badge);
  drawerBody.append(row);

  if (rule.formula) {
    drawerBody.append(section('Suffix formula', el('div', 'formula', rule.formula)));
  }
  drawerBody.append(section(rule.leaf ? 'Harmony & formation' : 'Overview', el('p', '', rule.harmony)));
  if (rule.notes) {
    drawerBody.append(section('Exceptions & notes', el('p', '', rule.notes)));
  }
  if (rule.examples.length) {
    const list = el('ul', 'examples');
    for (const ex of rule.examples) {
      const li = el('li');
      const tr = el('div', 'ex-tr');
      tr.lang = 'tr';
      tr.append(highlighted(ex.tr));
      li.append(tr, el('div', 'ex-en', ex.en));
      list.append(li);
    }
    drawerBody.append(section('Examples', list));
  }
  if (rule.children.length) {
    const chips = el('div', 'chips');
    rule.children.forEach((c) => chips.append(chip(c)));
    drawerBody.append(section(`Contains ${rule.children.length} ${rule.children.length === 1 ? 'entry' : 'entries'}`, chips));
  }

  drawerBody.scrollTop = 0;
  drawer.classList.add('open');
  drawer.setAttribute('aria-hidden', 'false');
  document.body.classList.add('drawer-open');
}

function closeInspector() {
  drawer.classList.remove('open');
  drawer.setAttribute('aria-hidden', 'true');
  document.body.classList.remove('drawer-open');
}

/** Chip that jumps to the grammar rule with the given Turkish title. */
function ruleLink(label, title) {
  const id = title ? app.rule_id_by_title(title) : -1;
  if (id < 0) return null;
  const c = el('button', 'chip', label);
  c.type = 'button';
  c.addEventListener('click', () => app.focus(id));
  return c;
}

function dictEntry(item) {
  const li = el('li');
  const row = el('button', 'dict-row');
  row.type = 'button';
  row.setAttribute('aria-expanded', 'false');
  const tr = el('span', 'dict-tr', item.tr);
  tr.lang = 'tr';
  row.append(el('span', 'dict-rank', `#${item.rank}`), tr, el('span', 'dict-en', item.en));

  const detail = el('div', 'dict-detail');
  const forms = el('dl', 'dict-forms');
  const addForm = (label, value) => { if (value) forms.append(el('dt', '', label), el('dd', '', value)); };
  const links = el('div', 'dict-links');
  const addLink = (label, title) => { const c = ruleLink(label, title); if (c) links.append(c); };
  if (dictKind === 'verbs') {
    addForm('Aorist', item.aorist);
    addForm('Continuous', item.cont);
    addForm('Takes', item.gov);
    addLink('Aorist rule', '-(A/I)r');
    addLink('-yor rule', '-(I)yor');
    item.gov.split(' / ').filter(Boolean).forEach((g) => addLink(`${g} case`, CASE_RULES[g]));
  } else {
    addForm('Opposite', item.antonym);
    addLink('Comparative', 'daha');
    addLink('Superlative', 'en');
  }
  detail.append(forms, links);

  row.addEventListener('click', () => {
    const open = li.classList.toggle('open');
    row.setAttribute('aria-expanded', String(open));
  });
  li.append(row, detail);
  return li;
}

function renderDict() {
  const q = dictInput.value;
  const res = JSON.parse(dict_search(dictKind, q));
  dictList.replaceChildren(...res.items.map(dictEntry));
  if (!res.items.length) dictList.append(el('li', 'dict-empty', 'No words match.'));
  dictCount.textContent = q.trim() ? `${res.items.length} of ${res.total}` : `${res.total}`;
  dictCount.classList.toggle('none', res.items.length === 0);
  dictList.scrollTop = 0;
}

function setDictOpen(open) {
  dict.classList.toggle('open', open);
  dict.setAttribute('aria-hidden', String(!open));
  dictButton.setAttribute('aria-expanded', String(open));
  document.body.classList.toggle('dict-open', open);
  if (open) {
    if (!dictList.children.length) renderDict();
    dictInput.focus();
  }
}

function local(e) {
  const r = canvas.getBoundingClientRect();
  return [e.clientX - r.left, e.clientY - r.top];
}

function resize() {
  app.resize(canvas.clientWidth, canvas.clientHeight, window.devicePixelRatio || 1);
}

function bind() {
  canvas.addEventListener('pointerdown', (e) => {
    if (e.button !== 0 && e.pointerType === 'mouse') return;
    canvas.setPointerCapture(e.pointerId);
    canvas.classList.add('dragging');
    app.pointer_down(e.pointerId, ...local(e));
  });
  canvas.addEventListener('pointermove', (e) => {
    const over = app.pointer_move(e.pointerId, ...local(e));
    canvas.classList.toggle('pointing', over);
  });
  const release = (e) => {
    canvas.classList.remove('dragging');
    if (canvas.hasPointerCapture(e.pointerId)) canvas.releasePointerCapture(e.pointerId);
  };
  canvas.addEventListener('pointerup', (e) => {
    release(e);
    app.pointer_up(e.pointerId, ...local(e));
  });
  canvas.addEventListener('pointercancel', (e) => {
    release(e);
    app.pointer_cancel(e.pointerId);
  });
  canvas.addEventListener('pointerleave', () => app.pointer_leave());
  canvas.addEventListener('dblclick', (e) => app.dblclick(...local(e)));
  canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    const unit = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? canvas.clientHeight : 1;
    // Trackpad pinch arrives as ctrl+wheel with small deltas; boost it.
    app.wheel(...local(e), e.deltaY * unit * (e.ctrlKey ? 4 : 1));
  }, { passive: false });

  window.addEventListener('resize', resize);
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      if (document.activeElement === dictInput && dictInput.value) {
        dictInput.value = '';
        renderDict();
        return;
      } else if (dict.classList.contains('open') && dict.contains(document.activeElement)) {
        setDictOpen(false);
        document.activeElement.blur();
        return;
      } else if (document.activeElement === searchInput && searchInput.value) {
        searchInput.value = '';
        runSearch();
      } else if (drawer.classList.contains('open')) {
        closeInspector();
        app.clear_selection();
      }
      searchInput.blur();
      return;
    }
    const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;
    if (typing || e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.key === '/' || e.key === 'f' || e.key === 'F') {
      e.preventDefault();
      searchInput.focus();
      searchInput.select();
    } else if (e.key === 'd' || e.key === 'D') {
      e.preventDefault();
      setDictOpen(!dict.classList.contains('open'));
    } else if (app.key(e.key)) {
      e.preventDefault();
    }
  });

  $('btn-expand').addEventListener('click', () => app.expand_all());
  $('btn-collapse').addEventListener('click', () => app.collapse_all());
  $('btn-reset').addEventListener('click', () => app.reset_camera());
  $('inspector-close').addEventListener('click', () => {
    closeInspector();
    app.clear_selection();
  });
  searchInput.addEventListener('input', runSearch);

  dictButton.addEventListener('click', () => setDictOpen(!dict.classList.contains('open')));
  $('dict-close').addEventListener('click', () => setDictOpen(false));
  dictInput.addEventListener('input', renderDict);
  for (const tab of dict.querySelectorAll('[role="tab"]')) {
    tab.addEventListener('click', () => {
      dictKind = tab.dataset.kind;
      dict.querySelectorAll('[role="tab"]').forEach((t) => t.setAttribute('aria-selected', String(t === tab)));
      renderDict();
    });
  }
}

function runSearch() {
  const q = searchInput.value;
  const count = app.search(q);
  if (!q.trim()) {
    searchCount.textContent = '';
  } else {
    searchCount.textContent = count === 1 ? '1 match' : `${count} matches`;
    searchCount.classList.toggle('none', count === 0);
  }
}

async function main() {
  try {
    await init();
    // Wait for the web fonts (at most 2.5 s) so label widths are measured
    // with the real typefaces; falls back to system fonts when offline.
    await Promise.race([
      Promise.all([
        document.fonts.load('500 13px Inter'),
        document.fonts.load('600 15px Inter'),
        document.fonts.load("500 12.5px 'JetBrains Mono'"),
      ]),
      new Promise((resolve) => setTimeout(resolve, 2500)),
    ]).catch(() => {});

    // <body data-view="radial"> is the topic map, "levels" the house of floors.
    app = new App(canvas, openInspector, document.body.dataset.view || 'radial');
    // Respect the system "reduce motion" setting: changes apply without easing.
    const motion = window.matchMedia('(prefers-reduced-motion: reduce)');
    app.set_reduced_motion(motion.matches);
    motion.addEventListener('change', (e) => app.set_reduced_motion(e.matches));
    // Console handle for debugging, e.g. grammarMap.focus(12).
    window.grammarMap = app;

    resize();
    bind();
    app.start();
    statusEl.classList.add('hidden');
  } catch (err) {
    console.error(err);
    statusEl.classList.add('error');
    statusEl.textContent =
      'Could not start the grammar engine.\n\n' +
      'Build it with:  wasm-pack build --target web --release\n' +
      'and serve this folder over http:// (not file://).\n\n' + err;
  }
}

main();
