// Shared interface motion for all pages: interruptible springs, drawers that
// can be flung closed, dialogs that materialise, and focus handling.
//
// Springs use Apple's two designer parameters: `response` (seconds, how fast
// it gets there) and `damping` (1 = no overshoot). Every animation starts
// from the value currently on screen, so it can be grabbed or reversed at any
// moment without a jump.

const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
const DECELERATION = 0.998;

/** Distance a fling travels before it stops (Apple's projection). */
export function project(velocity) {
  return (velocity / 1000) * DECELERATION / (1 - DECELERATION);
}

/**
 * A single animated number driven by a damped spring.
 * `set(target, velocity?)` re-targets from the current value and velocity.
 */
export class Spring {
  constructor(value, onUpdate, { response = 0.34, damping = 1 } = {}) {
    this.value = value;
    this.target = value;
    this.velocity = 0;
    this.onUpdate = onUpdate;
    this.response = response;
    this.damping = damping;
    this.running = false;
    this.onRest = null;
  }

  jump(value) {
    this.stop();
    this.value = this.target = value;
    this.velocity = 0;
    this.onUpdate(value);
  }

  set(target, velocity, onRest) {
    this.target = target;
    if (velocity !== undefined) this.velocity = velocity;
    this.onRest = onRest || null;
    if (reducedMotion.matches) {
      this.jump(target);
      onRest && onRest();
      return;
    }
    if (!this.running) {
      this.running = true;
      this.last = performance.now();
      requestAnimationFrame((t) => this.tick(t));
    }
  }

  stop() { this.running = false; }

  tick(now) {
    if (!this.running) return;
    const k = (2 * Math.PI / this.response) ** 2;
    const c = (4 * Math.PI * this.damping) / this.response;
    let dt = Math.min(0.064, (now - this.last) / 1000);
    this.last = now;
    // Sub-steps keep the integration stable on slow frames.
    while (dt > 0) {
      const h = Math.min(dt, 1 / 240);
      const a = -k * (this.value - this.target) - c * this.velocity;
      this.velocity += a * h;
      this.value += this.velocity * h;
      dt -= h;
    }
    const done = Math.abs(this.value - this.target) < 0.0005 * Math.max(1, Math.abs(this.target)) + 0.05 && Math.abs(this.velocity) < 1;
    if (done) {
      this.value = this.target;
      this.velocity = 0;
      this.running = false;
    }
    this.onUpdate(this.value);
    if (done) {
      const cb = this.onRest;
      this.onRest = null;
      cb && cb();
    } else {
      requestAnimationFrame((t) => this.tick(t));
    }
  }
}

// ───────────────────────────── focus ─────────────────────────────

const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function focusables(container) {
  return [...container.querySelectorAll(FOCUSABLE)].filter((e) => !e.closest('[hidden], [inert]') && e.offsetParent !== null);
}

/** Keeps Tab / Shift+Tab inside `container`. Call from a keydown handler. */
export function trapTab(container, e) {
  if (e.key !== 'Tab') return;
  const items = focusables(container);
  if (!items.length) { e.preventDefault(); return; }
  const first = items[0], last = items[items.length - 1];
  if (e.shiftKey && (document.activeElement === first || !container.contains(document.activeElement))) {
    e.preventDefault(); last.focus();
  } else if (!e.shiftKey && document.activeElement === last) {
    e.preventDefault(); first.focus();
  }
}

// ───────────────────────────── drawers ─────────────────────────────

const narrow = matchMedia('(max-width: 860px)');

/** Keeps `--hud-h` equal to the bottom of the top bar, so drawers sit under it. */
export function trackHudHeight() {
  const hud = document.querySelector('.hud');
  if (!hud) return;
  const update = () => document.documentElement.style.setProperty('--hud-h', `${Math.round(hud.getBoundingClientRect().bottom + 8)}px`);
  update();
  new ResizeObserver(update).observe(hud);
  addEventListener('resize', update);
}

/**
 * A side drawer that slides in from `side` ('left' or 'right') with a spring,
 * can be grabbed mid-flight, and on touch can be flung closed toward the side
 * it came from. On narrow screens it covers the page, so it gets a dimming
 * scrim and keeps keyboard focus inside.
 */
export class Drawer {
  constructor(el, { side = 'right', button = null, initialFocus = null, onClose = null, returnFocus = null } = {}) {
    this.el = el;
    this.returnFocus = returnFocus;
    this.side = side;
    this.button = button;
    this.initialFocus = initialFocus;
    this.onClose = onClose;
    this.isOpen = false;
    this.opener = null;
    this.scrim = document.createElement('div');
    this.scrim.className = 'scrim';
    this.scrim.hidden = true;
    el.before(this.scrim);
    this.scrim.addEventListener('click', () => this.close());
    this.spring = new Spring(1, (p) => this.render(p), { response: 0.36, damping: 1 });
    this.render(1);
    el.inert = true;
    this.bindDrag();
    el.addEventListener('keydown', (e) => { if (this.isOpen && narrow.matches) trapTab(el, e); });
  }

  /** 0 = open, 1 = closed (fraction of the width). */
  render(p) {
    const sign = this.side === 'right' ? 1 : -1;
    const fade = reducedMotion.matches;
    this.el.style.transform = fade ? 'none' : `translate3d(${sign * p * 105}%, 0, 0)`;
    this.el.style.opacity = fade ? String(1 - p) : '';
    this.el.style.visibility = p >= 0.999 && !this.isOpen ? 'hidden' : 'visible';
    const showScrim = narrow.matches && p < 0.999;
    this.scrim.hidden = !showScrim;
    this.scrim.style.opacity = String(0.55 * (1 - p));
  }

  width() { return this.el.getBoundingClientRect().width || 1; }

  open() {
    if (!this.isOpen) {
      this.opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    }
    this.isOpen = true;
    this.el.inert = false;
    this.el.style.visibility = 'visible'; // focusable right away, before the first frame
    this.el.classList.add('open');
    this.el.setAttribute('aria-hidden', 'false');
    this.button?.setAttribute('aria-expanded', 'true');
    this.spring.set(0);
    const target = (typeof this.initialFocus === 'function' ? this.initialFocus() : this.initialFocus) || focusables(this.el)[0];
    target?.focus({ preventScroll: true });
  }

  close() {
    if (!this.isOpen) return;
    this.isOpen = false;
    const hadFocus = this.el.contains(document.activeElement);
    this.el.classList.remove('open');
    this.el.setAttribute('aria-hidden', 'true');
    this.button?.setAttribute('aria-expanded', 'false');
    this.spring.set(1, undefined, () => { if (!this.isOpen) this.el.inert = true; });
    if (hadFocus) {
      const usable = (e) => e && e !== document.body && document.contains(e) && !this.el.contains(e);
      const back = [this.opener, this.returnFocus && this.returnFocus(), this.button].find(usable);
      back?.focus({ preventScroll: true });
    }
    this.onClose && this.onClose();
  }

  toggle() { this.isOpen ? this.close() : this.open(); }

  /** Touch / pen drag toward the edge it came from, with velocity handoff. */
  bindDrag() {
    let start = null, hist = [], grabbed = false;
    const sign = () => (this.side === 'right' ? 1 : -1);
    this.el.addEventListener('pointerdown', (e) => {
      if (e.pointerType === 'mouse' || !this.isOpen) return;
      start = { x: e.clientX, y: e.clientY, p: this.spring.value, id: e.pointerId };
      hist = [[performance.now(), e.clientX]];
      grabbed = false;
    });
    this.el.addEventListener('pointermove', (e) => {
      if (!start || e.pointerId !== start.id) return;
      const dx = e.clientX - start.x, dy = e.clientY - start.y;
      if (!grabbed) {
        // About 10 px of hysteresis, and only for a mostly horizontal move.
        if (Math.hypot(dx, dy) < 10) return;
        if (Math.abs(dx) < Math.abs(dy) * 1.2) { start = null; return; }
        grabbed = true;
        this.spring.stop();
        this.el.setPointerCapture(e.pointerId);
      }
      hist.push([performance.now(), e.clientX]);
      if (hist.length > 6) hist.shift();
      const p = Math.max(0, start.p + (sign() * dx) / this.width());
      this.spring.jump(Math.min(1, p));
    });
    const end = (e) => {
      if (!start || e.pointerId !== start.id) return;
      const was = grabbed;
      start = null;
      grabbed = false;
      if (!was) return;
      const [t0, x0] = hist[0], [t1, x1] = hist[hist.length - 1];
      const v = t1 > t0 && performance.now() - t1 < 80 ? ((x1 - x0) / (t1 - t0)) * 1000 : 0; // px/s
      const projected = this.spring.value + (sign() * project(v)) / this.width();
      const velocity = (sign() * v) / this.width();
      if (projected > 0.4) {
        this.isOpen = true; // let close() run its bookkeeping
        this.spring.velocity = velocity;
        this.close();
      } else {
        this.spring.set(0, velocity);
      }
    };
    this.el.addEventListener('pointerup', end);
    this.el.addEventListener('pointercancel', end);
    narrow.addEventListener('change', () => this.render(this.spring.value));
  }
}

// ───────────────────────────── dialogs & popovers ─────────────────────────────

/**
 * Makes `el` materialise (scale + blur + opacity on one critically damped
 * spring) when shown, and dissolve the same way when hidden. `origin` sets
 * where it grows from, e.g. the word that opened it.
 */
export class Materialise {
  constructor(el, { onHidden = null } = {}) {
    this.el = el;
    this.onHidden = onHidden;
    this.spring = new Spring(0, (p) => this.render(p), { response: 0.3, damping: 1 });
  }

  render(p) {
    const q = Math.max(0, Math.min(1, p));
    const fade = reducedMotion.matches;
    this.el.style.opacity = String(q);
    this.el.style.transform = fade ? '' : `scale(${0.94 + 0.06 * q})`;
    this.el.style.filter = fade || q > 0.995 ? '' : `blur(${(1 - q) * 6}px)`;
  }

  show(origin) {
    if (origin) this.el.style.transformOrigin = origin;
    this.el.hidden = false;
    this.spring.set(1);
  }

  hide() {
    this.spring.set(0, undefined, () => {
      this.el.hidden = true;
      this.onHidden && this.onHidden();
    });
  }
}

/** Modal dialog: scrim + materialising card, focus kept inside, restored on close. */
export class Modal {
  constructor(el) {
    this.el = el;
    this.card = el.querySelector('.modal-card') || el.firstElementChild;
    this.opener = null;
    this.backdrop = new Spring(0, (p) => { el.style.opacity = String(Math.max(0, Math.min(1, p))); }, { response: 0.28, damping: 1 });
    this.card_ = new Materialise(this.card);
    el.addEventListener('keydown', (e) => trapTab(el, e));
  }

  get isOpen() { return !this.el.hidden && this.el.dataset.closing !== '1'; }

  open() {
    this.opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    delete this.el.dataset.closing;
    this.el.hidden = false;
    this.backdrop.set(1);
    this.card_.show('50% 40%');
    const first = this.el.querySelector('input:not([type=radio]), select, textarea') || focusables(this.el)[0];
    first?.focus({ preventScroll: true });
  }

  close() {
    if (this.el.hidden) return;
    this.el.dataset.closing = '1';
    this.card_.hide();
    this.backdrop.set(0, undefined, () => {
      if (this.el.dataset.closing === '1') { this.el.hidden = true; delete this.el.dataset.closing; }
    });
    this.opener?.focus({ preventScroll: true });
  }
}
