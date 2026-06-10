import * as fs from "fs";
import { fileURLToPath } from "url";

// Variable ref (not a string literal) so Vite does not inline the .wasm as a data URL.
const _wasmRel = "../polyhook.wasm";

// ---------------------------------------------------------------------------
// Types (re-exported from generated/types — source of truth is schema.json)
// ---------------------------------------------------------------------------

export type { CallerKind, HookEvent, HookResponse } from "./generated/types";

import type { CallerKind, HookEvent, HookResponse } from "./generated/types";

/** Approve the action without modification. */
export interface ApproveResponse {
  action: "approve";
}

/** Block the action, surfacing a message to the user / agent. */
export interface BlockResponse {
  action: "block";
  message: string;
}

/** Approve the action but replace the input with modified fields. */
export interface ModifyResponse {
  action: "modify";
  input: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Internal WASM state (lazy-initialised once per process)
// ---------------------------------------------------------------------------

/**
 * Structural type for the WASM memory object.
 * Avoids depending on the DOM lib's WebAssembly namespace — Node.js exposes
 * the same object at runtime but TypeScript only types it in lib.dom.d.ts.
 */
interface WasmMemory {
  readonly buffer: ArrayBuffer;
}

export interface WasmExports {
  memory: WasmMemory;
  alloc(len: number): number;
  dealloc(ptr: number, len: number): void;
  parse(ptr: number, len: number): number;
  serialize(ptr: number, len: number): number;
}

let _wasm: WasmExports | null = null;

// Kept across read() → respond() within a single hook invocation so that
// respond() can serialise the caller information back out.
let _lastCaller: CallerKind = "unknown";

// Allow tests to inject a mock instance without touching the filesystem.
export function _setWasmInstance(instance: WasmExports | null): void {
  _wasm = instance;
}

async function getWasm(): Promise<WasmExports> {
  if (_wasm !== null) return _wasm;

  const wasmPath = fileURLToPath(new URL(_wasmRel, import.meta.url));
  const wasmBytes = fs.readFileSync(wasmPath);
  // Access the global WebAssembly object via globalThis so TypeScript does not
  // require the DOM lib (where the WebAssembly namespace is declared).
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const wa = (globalThis as any).WebAssembly as {
    instantiate(
      bytes: ArrayBuffer | Uint8Array,
    ): Promise<{ instance: { exports: unknown } }>;
  };
  const { instance } = await wa.instantiate(wasmBytes);
  _wasm = instance.exports as WasmExports;
  return _wasm;
}

// ---------------------------------------------------------------------------
// Memory helpers
// ---------------------------------------------------------------------------

/**
 * Write `bytes` into WASM memory starting at `ptr`.
 */
function writeBytes(wasm: WasmExports, ptr: number, bytes: Uint8Array): void {
  const mem = new Uint8Array(wasm.memory.buffer);
  mem.set(bytes, ptr);
}

/**
 * Read a length-prefixed payload from WASM memory.
 * Layout: 4 bytes LE i32 = payload length, then `length` payload bytes.
 * Returns the payload as a Uint8Array (a copy, safe after dealloc).
 */
function readLengthPrefixed(wasm: WasmExports, ptr: number): Uint8Array {
  const dv = new DataView(wasm.memory.buffer);
  const len = dv.getInt32(ptr, /* littleEndian */ true);
  const payload = new Uint8Array(wasm.memory.buffer, ptr + 4, len);
  // Return a copy so the caller can dealloc before using the data.
  return Uint8Array.from(payload);
}

// ---------------------------------------------------------------------------
// stdin helper
// ---------------------------------------------------------------------------

async function readStdin(): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    process.stdin.on("data", (c: Buffer) => chunks.push(c));
    process.stdin.on("end", () => resolve(Buffer.concat(chunks)));
    process.stdin.on("error", (err: Error) => reject(err));
  });
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Read and parse the hook event delivered on stdin.
 *
 * The raw bytes are fed through the WASM `parse` function which normalises
 * the caller-specific wire format into a unified {@link HookEvent} shape.
 *
 * Call this **once** at the start of your hook handler.
 */
export async function read(): Promise<HookEvent> {
  const wasm = await getWasm();

  const inputBuf = await readStdin();
  const inputBytes = new Uint8Array(
    inputBuf.buffer,
    inputBuf.byteOffset,
    inputBuf.byteLength,
  );
  const inputLen = inputBytes.length;

  // Allocate space in WASM memory and copy the input bytes in.
  const inputPtr = wasm.alloc(inputLen);
  writeBytes(wasm, inputPtr, inputBytes);

  // Parse — the WASM function returns a pointer to a length-prefixed JSON blob.
  const resultPtr = wasm.parse(inputPtr, inputLen);

  // Read the result before freeing.
  const resultBytes = readLengthPrefixed(wasm, resultPtr);
  const resultLen = new DataView(wasm.memory.buffer).getInt32(resultPtr, true);

  // Free WASM allocations.
  wasm.dealloc(resultPtr, 4 + resultLen);
  wasm.dealloc(inputPtr, inputLen);

  const json = new TextDecoder().decode(resultBytes);
  const event = JSON.parse(json) as HookEvent;

  // Cache the caller so respond() can include it without the caller having
  // to thread it through.
  _lastCaller = event.caller ?? "unknown";

  return event;
}

/**
 * Serialise and write a {@link HookResponse} to stdout.
 *
 * The WASM `serialize` function converts the response into the wire format
 * expected by the calling agent.
 *
 * Call this **once** after processing the event returned by {@link read}.
 */
export async function respond(r: HookResponse): Promise<void> {
  const wasm = await getWasm();

  const json = JSON.stringify(r);
  const inputBytes = new TextEncoder().encode(json);
  const inputLen = inputBytes.length;

  // Allocate, copy, call serialize.
  const inputPtr = wasm.alloc(inputLen);
  writeBytes(wasm, inputPtr, inputBytes);

  const resultPtr = wasm.serialize(inputPtr, inputLen);

  // Read the result before freeing.
  const resultBytes = readLengthPrefixed(wasm, resultPtr);
  const resultLen = new DataView(wasm.memory.buffer).getInt32(resultPtr, true);

  // Free WASM allocations.
  wasm.dealloc(resultPtr, 4 + resultLen);
  wasm.dealloc(inputPtr, inputLen);

  // Write the serialised response to stdout.
  await new Promise<void>((resolve, reject) => {
    process.stdout.write(resultBytes, (err) => {
      if (err) reject(err);
      else resolve();
    });
  });
}

// ---------------------------------------------------------------------------
// Convenience helpers
// ---------------------------------------------------------------------------

/** Return an {@link ApproveResponse} object (does NOT call respond). */
export function approve(): ApproveResponse {
  return { action: "approve" };
}

/** Return a {@link BlockResponse} object (does NOT call respond). */
export function block(message: string): BlockResponse {
  return { action: "block", message };
}

/**
 * Return a {@link ModifyResponse} object (does NOT call respond).
 *
 * @param input  Replacement fields that the agent should use instead of the
 *               original input.
 */
export function modify(input: Record<string, unknown>): ModifyResponse {
  return { action: "modify", input };
}
