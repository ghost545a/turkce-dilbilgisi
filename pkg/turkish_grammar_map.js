/* @ts-self-types="./turkish_grammar_map.d.ts" */

/**
 * Handle exported to JavaScript.
 */
export class App {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AppFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_app_free(ptr, 0);
    }
    clear_selection() {
        wasm.app_clear_selection(this.__wbg_ptr);
    }
    collapse_all() {
        wasm.app_collapse_all(this.__wbg_ptr);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    dblclick(x, y) {
        wasm.app_dblclick(this.__wbg_ptr, x, y);
    }
    expand_all() {
        wasm.app_expand_all(this.__wbg_ptr);
    }
    /**
     * @param {number} id
     */
    focus(id) {
        wasm.app_focus(this.__wbg_ptr, id);
    }
    /**
     * Returns `true` if the key was handled.
     * @param {string} key
     * @returns {boolean}
     */
    key(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.app_key(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * `on_inspect(json)` is called whenever a rule should open in the drawer.
     * @param {HTMLCanvasElement} canvas
     * @param {Function} on_inspect
     */
    constructor(canvas, on_inspect) {
        const ret = wasm.app_new(canvas, on_inspect);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        AppFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    node_count() {
        const ret = wasm.app_node_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} id
     * @returns {string}
     */
    node_json(id) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.app_node_json(this.__wbg_ptr, id);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} id
     */
    pointer_cancel(id) {
        wasm.app_pointer_cancel(this.__wbg_ptr, id);
    }
    /**
     * @param {number} id
     * @param {number} x
     * @param {number} y
     */
    pointer_down(id, x, y) {
        wasm.app_pointer_down(this.__wbg_ptr, id, x, y);
    }
    pointer_leave() {
        wasm.app_pointer_leave(this.__wbg_ptr);
    }
    /**
     * @param {number} id
     * @param {number} x
     * @param {number} y
     * @returns {boolean}
     */
    pointer_move(id, x, y) {
        const ret = wasm.app_pointer_move(this.__wbg_ptr, id, x, y);
        return ret !== 0;
    }
    /**
     * @param {number} id
     * @param {number} x
     * @param {number} y
     */
    pointer_up(id, x, y) {
        wasm.app_pointer_up(this.__wbg_ptr, id, x, y);
    }
    reset_camera() {
        wasm.app_reset_camera(this.__wbg_ptr);
    }
    /**
     * @param {number} width
     * @param {number} height
     * @param {number} dpr
     */
    resize(width, height, dpr) {
        wasm.app_resize(this.__wbg_ptr, width, height, dpr);
    }
    /**
     * Id of the first rule whose Turkish title is exactly `title`, or -1.
     * @param {string} title
     * @returns {number}
     */
    rule_id_by_title(title) {
        const ptr0 = passStringToWasm0(title, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.app_rule_id_by_title(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Apply a search query; returns the number of matching rules.
     * @param {string} query
     * @returns {number}
     */
    search(query) {
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.app_search(this.__wbg_ptr, ptr0, len0);
        return ret >>> 0;
    }
    /**
     * Start the `requestAnimationFrame` loop (idempotent).
     */
    start() {
        const ret = wasm.app_start(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} delta_y
     */
    wheel(x, y, delta_y) {
        wasm.app_wheel(this.__wbg_ptr, x, y, delta_y);
    }
}
if (Symbol.dispose) App.prototype[Symbol.dispose] = App.prototype.free;

/**
 * Frequency dictionary lookup. `kind` is "verbs" or "adjectives"; returns
 * JSON `{kind, total, items}` with the matching entries, most common first.
 * @param {string} kind
 * @param {string} query
 * @returns {string}
 */
export function dict_search(kind, query) {
    let deferred3_0;
    let deferred3_1;
    try {
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.dict_search(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_undefined_be5b34249ff1fc15: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_4c88d8bf264fd984: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_09736879a3f67559: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_addColorStop_5062bdb863ddbc7c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addColorStop(arg1, getStringFromWasm0(arg2, arg3));
        }, arguments); },
        __wbg_arc_df3f7eb77e59876f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.arc(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_beginPath_ab690dc0c7efe6dc: function(arg0) {
            arg0.beginPath();
        },
        __wbg_bezierCurveTo_fcaa1adffbe3419a: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.bezierCurveTo(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_call_a410eb4b59ed5965: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_closePath_e9e11e034c5d75a3: function(arg0) {
            arg0.closePath();
        },
        __wbg_createRadialGradient_d60344cf1fd67bbf: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            const ret = arg0.createRadialGradient(arg1, arg2, arg3, arg4, arg5, arg6);
            return ret;
        }, arguments); },
        __wbg_fillRect_e1f0b9915220348b: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fillText_89d353a9ad9f4a1a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_fill_cb36fe21a8d46cd7: function(arg0) {
            arg0.fill();
        },
        __wbg_getContext_d93f904b714852ae: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_instanceof_CanvasRenderingContext2d_72a60be8aaadecab: function(arg0) {
            let result;
            try {
                result = arg0 instanceof CanvasRenderingContext2D;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_5cf453f5639aa1a2: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_measureText_d831259b682a3f7d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.measureText(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_moveTo_b4cd3b0b7e1585a5: function(arg0, arg1, arg2) {
            arg0.moveTo(arg1, arg2);
        },
        __wbg_requestAnimationFrame_9ba49d36a19ca83c: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            return ret;
        }, arguments); },
        __wbg_restore_f322d9dcee2d3472: function(arg0) {
            arg0.restore();
        },
        __wbg_rotate_c33a3db69ad81687: function() { return handleError(function (arg0, arg1) {
            arg0.rotate(arg1);
        }, arguments); },
        __wbg_save_1375b4c2f3596b8b: function(arg0) {
            arg0.save();
        },
        __wbg_scale_57e5c903fb153a90: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.scale(arg1, arg2);
        }, arguments); },
        __wbg_setTransform_ee8f5d076fcdec0f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setTransform(arg1, arg2, arg3, arg4, arg5, arg6);
        }, arguments); },
        __wbg_set_fillStyle_94e9d6bece177fdd: function(arg0, arg1) {
            arg0.fillStyle = arg1;
        },
        __wbg_set_fillStyle_95fde67582a9c695: function(arg0, arg1, arg2) {
            arg0.fillStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_font_aa0c0c514c73a5fa: function(arg0, arg1, arg2) {
            arg0.font = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_globalAlpha_4bdefe001e9c79ce: function(arg0, arg1) {
            arg0.globalAlpha = arg1;
        },
        __wbg_set_height_ae0610d318c3428c: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_lineCap_e73c68b48b40a360: function(arg0, arg1, arg2) {
            arg0.lineCap = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineWidth_12f039d09cd36dad: function(arg0, arg1) {
            arg0.lineWidth = arg1;
        },
        __wbg_set_shadowBlur_85cdec581d1dc1d5: function(arg0, arg1) {
            arg0.shadowBlur = arg1;
        },
        __wbg_set_shadowColor_b938633d4fa98064: function(arg0, arg1, arg2) {
            arg0.shadowColor = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_strokeStyle_b90c8a6e0d37c80d: function(arg0, arg1, arg2) {
            arg0.strokeStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textAlign_0a32c51401892c11: function(arg0, arg1, arg2) {
            arg0.textAlign = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textBaseline_ff2e3c5c84684b23: function(arg0, arg1, arg2) {
            arg0.textBaseline = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_width_5f89a551b5472887: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_static_accessor_GLOBAL_THIS_003ee833af9699fc: function() {
            const ret = typeof globalThis === 'undefined' ? undefined : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_e4293fb22cbe77da: function() {
            const ret = typeof global === 'undefined' ? undefined : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_53bb20457401453d: function() {
            const ret = typeof self === 'undefined' ? undefined : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_b5e0810bfecb7feb: function() {
            const ret = typeof window === 'undefined' ? undefined : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_stroke_19fa3f3ec093bc25: function(arg0) {
            arg0.stroke();
        },
        __wbg_translate_73a88129c5d9f3b2: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.translate(arg1, arg2);
        }, arguments); },
        __wbg_width_28f5533f1b3b5abc: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [F64], shim_idx: 3, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_221b4da4ce97537e___convert__closures_____invoke___f64______true_);
            return ret;
        },
        __wbindgen_generic_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            table.set(0, undefined);
            table.set(1024, undefined);
            table.set(1025, null);
            table.set(1026, true);
            table.set(1027, false);
        },
    };
    return {
        __proto__: null,
        "./turkish_grammar_map_bg.js": import0,
    };
}

function wasm_bindgen_221b4da4ce97537e___convert__closures_____invoke___f64______true_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_221b4da4ce97537e___convert__closures_____invoke___f64______true_(arg0, arg1, arg2);
}

const AppFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_app_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(bytes) {
    numBytesDecoded += bytes.length;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
        cachedTextDecoder.decode();
        numBytesDecoded = bytes.length;
    }
    return cachedTextDecoder.decode(bytes);
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

let __wbg_init_promise;
async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;
    if (__wbg_init_promise === undefined) {
        __wbg_init_promise = __wbg_init_once(module_or_path);
        __wbg_init_promise.catch(() => { __wbg_init_promise = undefined; });
    }
    return __wbg_init_promise;
}

async function __wbg_init_once(module_or_path) {

    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('turkish_grammar_map_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
