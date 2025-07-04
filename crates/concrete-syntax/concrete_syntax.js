
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextEncoder, TextDecoder } = require(`util`);

let WASM_VECTOR_LEN = 0;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

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
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_3.set(idx, obj);
    return idx;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_3.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
 * Standalone function for direct pattern matching without creating a ConcreteSyntaxMatcher instance
 * @param {string} pattern_str
 * @param {WasmSyntaxNode} node
 * @param {Uint8Array} source_code
 * @param {boolean} recursive
 * @param {string | null} [replace_node]
 * @returns {any}
 */
module.exports.getAllMatches = function(pattern_str, node, source_code, recursive, replace_node) {
    const ptr0 = passStringToWasm0(pattern_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(source_code, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    var ptr2 = isLikeNone(replace_node) ? 0 : passStringToWasm0(replace_node, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len2 = WASM_VECTOR_LEN;
    const ret = wasm.getAllMatches(ptr0, len0, node, ptr1, len1, recursive, ptr2, len2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
};

/**
 * Parse a concrete syntax pattern and return it as a serialized object for inspection
 * @param {string} pattern_str
 * @returns {any}
 */
module.exports.parsePattern = function(pattern_str) {
    const ptr0 = passStringToWasm0(pattern_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.parsePattern(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
};

const ConcreteSyntaxMatcherFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_concretesyntaxmatcher_free(ptr >>> 0, 1));
/**
 * WASM-compatible interface for the concrete syntax matcher
 */
class ConcreteSyntaxMatcher {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ConcreteSyntaxMatcherFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_concretesyntaxmatcher_free(ptr, 0);
    }
    /**
     * @param {string} pattern_str
     */
    constructor(pattern_str) {
        const ptr0 = passStringToWasm0(pattern_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.concretesyntaxmatcher_new(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        ConcreteSyntaxMatcherFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {WasmSyntaxNode} node
     * @param {Uint8Array} source_code
     * @param {boolean} recursive
     * @returns {any}
     */
    findMatches(node, source_code, recursive) {
        const ptr0 = passArray8ToWasm0(source_code, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.concretesyntaxmatcher_findMatches(this.__wbg_ptr, node, ptr0, len0, recursive);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
module.exports.ConcreteSyntaxMatcher = ConcreteSyntaxMatcher;

module.exports.__wbg_String_8f0eb39a4a4c2f66 = function(arg0, arg1) {
    const ret = String(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbg_childCount_6b5c8a1ab88a8f9a = function(arg0) {
    const ret = arg0.childCount;
    return ret;
};

module.exports.__wbg_child_343cc41b36376a1a = function(arg0, arg1) {
    const ret = arg0.child(arg1 >>> 0);
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_children_404eb2809194c3eb = function(arg0) {
    const ret = arg0.children();
    return ret;
};

module.exports.__wbg_clone_9fbfa48f4cff6442 = function(arg0) {
    const ret = arg0.clone();
    return ret;
};

module.exports.__wbg_clone_e68c4ec511b65289 = function(arg0) {
    const ret = arg0.clone();
    return ret;
};

module.exports.__wbg_endByte_f337e696964687ec = function(arg0) {
    const ret = arg0.endByte;
    return ret;
};

module.exports.__wbg_endColumn_186e95d583866e5d = function(arg0) {
    const ret = arg0.endColumn;
    return ret;
};

module.exports.__wbg_endRow_4110de6a009d6e50 = function(arg0) {
    const ret = arg0.endRow;
    return ret;
};

module.exports.__wbg_get_b9b93047fe3cf45b = function(arg0, arg1) {
    const ret = arg0[arg1 >>> 0];
    return ret;
};

module.exports.__wbg_gotoFirstChild_76db0e246cad24cc = function(arg0) {
    const ret = arg0.gotoFirstChild();
    return ret;
};

module.exports.__wbg_gotoNextSibling_1634d8dcad7239e2 = function(arg0) {
    const ret = arg0.gotoNextSibling();
    return ret;
};

module.exports.__wbg_gotoParent_43708743818dfffc = function(arg0) {
    const ret = arg0.gotoParent();
    return ret;
};

module.exports.__wbg_kind_8b15881157c80867 = function(arg0, arg1) {
    const ret = arg1.kind;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbg_length_e2d2a49132c1b256 = function(arg0) {
    const ret = arg0.length;
    return ret;
};

module.exports.__wbg_log_c222819a41e063d3 = function(arg0) {
    console.log(arg0);
};

module.exports.__wbg_new_405e22f390576ce2 = function() {
    const ret = new Object();
    return ret;
};

module.exports.__wbg_new_5e0be73521bc8c17 = function() {
    const ret = new Map();
    return ret;
};

module.exports.__wbg_new_78feb108b6472713 = function() {
    const ret = new Array();
    return ret;
};

module.exports.__wbg_node_ed870f6583dbcb67 = function(arg0) {
    const ret = arg0.node();
    return ret;
};

module.exports.__wbg_set_37837023f3d740e8 = function(arg0, arg1, arg2) {
    arg0[arg1 >>> 0] = arg2;
};

module.exports.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
};

module.exports.__wbg_set_8fc6bf8a5b1071d1 = function(arg0, arg1, arg2) {
    const ret = arg0.set(arg1, arg2);
    return ret;
};

module.exports.__wbg_startByte_919fcb5d1ba64ae7 = function(arg0) {
    const ret = arg0.startByte;
    return ret;
};

module.exports.__wbg_startColumn_24646ec938575291 = function(arg0) {
    const ret = arg0.startColumn;
    return ret;
};

module.exports.__wbg_startRow_36b042d63ef35a0a = function(arg0) {
    const ret = arg0.startRow;
    return ret;
};

module.exports.__wbg_walk_3599a8622cdfb365 = function(arg0) {
    const ret = arg0.walk();
    return ret;
};

module.exports.__wbindgen_as_number = function(arg0) {
    const ret = +arg0;
    return ret;
};

module.exports.__wbindgen_bigint_from_u64 = function(arg0) {
    const ret = BigInt.asUintN(64, arg0);
    return ret;
};

module.exports.__wbindgen_error_new = function(arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

module.exports.__wbindgen_init_externref_table = function() {
    const table = wasm.__wbindgen_export_3;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

module.exports.__wbindgen_is_string = function(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
};

module.exports.__wbindgen_number_new = function(arg0) {
    const ret = arg0;
    return ret;
};

module.exports.__wbindgen_string_new = function(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

const path = require('path').join(__dirname, 'concrete_syntax_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

wasm.__wbindgen_start();

