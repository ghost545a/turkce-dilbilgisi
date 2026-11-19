/* tslint:disable */
/* eslint-disable */

/**
 * Handle exported to JavaScript.
 */
export class App {
    free(): void;
    [Symbol.dispose](): void;
    clear_selection(): void;
    collapse_all(): void;
    dblclick(x: number, y: number): void;
    expand_all(): void;
    focus(id: number): void;
    /**
     * Screen position of the ⓘ button, if shown (testing aid).
     */
    info_screen_pos(): Float64Array;
    /**
     * Returns `true` if the key was handled.
     */
    key(key: string): boolean;
    /**
     * Changes whenever the visible tree changes (for the keyboard tree).
     */
    layout_version(): number;
    /**
     * `on_inspect(json)` is called whenever a rule should open in the drawer.
     * `view` is `"levels"` for the house-of-floors view, anything else for
     * the radial map.
     */
    constructor(canvas: HTMLCanvasElement, on_inspect: Function, view: string);
    node_count(): number;
    node_json(id: number): string;
    pointer_cancel(id: number): void;
    pointer_down(id: number, x: number, y: number): void;
    pointer_leave(): void;
    pointer_move(id: number, x: number, y: number): boolean;
    pointer_up(id: number, x: number, y: number): void;
    reset_camera(): void;
    resize(width: number, height: number, dpr: number): void;
    /**
     * Id of the first rule whose Turkish title is exactly `title`, or -1.
     */
    rule_id_by_title(title: string): number;
    /**
     * Apply a search query; returns the number of matching rules.
     */
    search(query: string): number;
    /**
     * Highlights a node and brings it into view (keyboard navigation).
     */
    select(id: number): void;
    /**
     * Opens or closes a branch.
     */
    set_expanded(id: number, expanded: boolean): void;
    /**
     * Turn animations off (`true`) or on. With animations off every change
     * is applied on the next frame without easing.
     */
    set_reduced_motion(reduced: boolean): void;
    /**
     * Multiplies every map label size (1 = normal). Re-lays out the map.
     */
    set_text_scale(scale: number): void;
    /**
     * Height of the page's top bar, kept clear when the view is fitted.
     */
    set_top_inset(px: number): void;
    /**
     * Whether the latest input came from a touch screen (larger hit areas).
     */
    set_touch(touch: boolean): void;
    /**
     * Start the `requestAnimationFrame` loop (idempotent).
     */
    start(): void;
    /**
     * Visible nodes in reading order, as JSON
     * `[{id, parent, depth, tr, en, branch, expanded, count}]`.
     */
    tree_json(): string;
    wheel(x: number, y: number, delta_y: number): void;
    /**
     * Wheel with both axes: on the cards page it scrolls (Ctrl zooms);
     * on the radial map it zooms at the cursor.
     */
    wheel_ex(x: number, y: number, delta_x: number, delta_y: number, ctrl: boolean): void;
}

/**
 * Frequency dictionary lookup. `kind` is "verbs" or "adjectives"; returns
 * JSON `{kind, total, items}` with the matching entries, most common first.
 * Turkish-English dictionary for the `/player/` page, built from the TSV the
 * page fetches (`player/tr_en_dictionary.tsv`).
 */
export class PlayerDictionary {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * JSON `{word, translation, note}` for a clicked word, or `null`.
     */
    lookup(word: string): string;
    constructor(tsv: string);
    /**
     * Number of entries loaded.
     */
    size(): number;
}

export function dict_search(kind: string, query: string): string;

/**
 * Parses a Turkish and an (optional) English subtitle file and pairs them.
 * Returns JSON `{ok, error, cues: [{id, start, end, tr, en}], warnings}`.
 */
export function parse_dual_srt(turkish: string, english: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_app_free: (a: number, b: number) => void;
    readonly __wbg_playerdictionary_free: (a: number, b: number) => void;
    readonly app_clear_selection: (a: number) => void;
    readonly app_collapse_all: (a: number) => void;
    readonly app_dblclick: (a: number, b: number, c: number) => void;
    readonly app_expand_all: (a: number) => void;
    readonly app_focus: (a: number, b: number) => void;
    readonly app_info_screen_pos: (a: number) => [number, number];
    readonly app_key: (a: number, b: number, c: number) => number;
    readonly app_layout_version: (a: number) => number;
    readonly app_new: (a: any, b: any, c: number, d: number) => [number, number, number];
    readonly app_node_count: (a: number) => number;
    readonly app_node_json: (a: number, b: number) => [number, number];
    readonly app_pointer_cancel: (a: number, b: number) => void;
    readonly app_pointer_down: (a: number, b: number, c: number, d: number) => void;
    readonly app_pointer_leave: (a: number) => void;
    readonly app_pointer_move: (a: number, b: number, c: number, d: number) => number;
    readonly app_pointer_up: (a: number, b: number, c: number, d: number) => void;
    readonly app_reset_camera: (a: number) => void;
    readonly app_resize: (a: number, b: number, c: number, d: number) => void;
    readonly app_rule_id_by_title: (a: number, b: number, c: number) => number;
    readonly app_search: (a: number, b: number, c: number) => number;
    readonly app_select: (a: number, b: number) => void;
    readonly app_set_expanded: (a: number, b: number, c: number) => void;
    readonly app_set_reduced_motion: (a: number, b: number) => void;
    readonly app_set_text_scale: (a: number, b: number) => void;
    readonly app_set_top_inset: (a: number, b: number) => void;
    readonly app_set_touch: (a: number, b: number) => void;
    readonly app_start: (a: number) => [number, number];
    readonly app_tree_json: (a: number) => [number, number];
    readonly app_wheel: (a: number, b: number, c: number, d: number) => void;
    readonly app_wheel_ex: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly dict_search: (a: number, b: number, c: number, d: number) => [number, number];
    readonly parse_dual_srt: (a: number, b: number, c: number, d: number) => [number, number];
    readonly playerdictionary_lookup: (a: number, b: number, c: number) => [number, number];
    readonly playerdictionary_new: (a: number, b: number) => number;
    readonly playerdictionary_size: (a: number) => number;
    readonly wasm_bindgen_221b4da4ce97537e___convert__closures_____invoke___f64______true_: (a: number, b: number, c: number) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
