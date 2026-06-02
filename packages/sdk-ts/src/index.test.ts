/**
 * Unit tests for @polyhook/sdk
 *
 * The WASM module is replaced with an in-process mock so tests run without a
 * compiled polyhook.wasm binary.  The mock implements the same memory-layout
 * contract:
 *   - alloc(len) → ptr  — allocates into a shared ArrayBuffer
 *   - dealloc(ptr, len) — no-op in the mock (GC handles it)
 *   - parse(ptr, len) → result_ptr  — decodes caller-specific JSON, returns
 *                                     normalised HookEvent as length-prefixed blob
 *   - serialize(ptr, len) → result_ptr — echoes the response JSON back as a
 *                                        length-prefixed blob (agent wire format
 *                                        is pass-through in tests)
 */

import { vi } from 'vitest'
import { read, respond, approve, block, modify, _setWasmInstance, WasmExports, HookEvent, HookResponse, CallerKind } from './index'

// Mock the 'fs' module so we can control readFileSync behaviour.
vi.mock('fs')
import * as fs from 'fs'

// ---------------------------------------------------------------------------
// Mock WASM factory
// ---------------------------------------------------------------------------

const HEAP_SIZE = 1024 * 1024 // 1 MiB — plenty for test payloads

function buildMockWasm(
  parseImpl: (inputJson: string) => HookEvent,
  serializeImpl: (response: HookResponse) => string = (r) => JSON.stringify(r),
): WasmExports {
  const buf = new ArrayBuffer(HEAP_SIZE)
  const memory = { buffer: buf }
  let cursor = 4 // reserve the first 4 bytes so ptr=0 acts as null

  function alloc(len: number): number {
    const ptr = cursor
    cursor += len
    return ptr
  }

  function dealloc(_ptr: number, _len: number): void {
    // No-op in tests — we use a bump allocator and let the GC clean up.
  }

  function writeLengthPrefixed(json: string): number {
    const encoded = new TextEncoder().encode(json)
    const totalLen = 4 + encoded.length
    const ptr = alloc(totalLen)
    const dv = new DataView(memory.buffer)
    dv.setInt32(ptr, encoded.length, /* littleEndian */ true)
    new Uint8Array(memory.buffer).set(encoded, ptr + 4)
    return ptr
  }

  function readString(ptr: number, len: number): string {
    const bytes = new Uint8Array(memory.buffer, ptr, len)
    return new TextDecoder().decode(bytes)
  }

  function parse(ptr: number, len: number): number {
    const inputJson = readString(ptr, len)
    const event = parseImpl(inputJson)
    return writeLengthPrefixed(JSON.stringify(event))
  }

  function serialize(ptr: number, len: number): number {
    const inputJson = readString(ptr, len)
    const response = JSON.parse(inputJson) as HookResponse
    const outJson = serializeImpl(response)
    return writeLengthPrefixed(outJson)
  }

  return { memory, alloc, dealloc, parse, serialize }
}

// ---------------------------------------------------------------------------
// stdin / stdout helpers
// ---------------------------------------------------------------------------

/** Replace process.stdin with a Readable that immediately emits `data`. */
function mockStdin(data: string): void {
  const { Readable } = require('stream')
  const readable = new Readable({ read() {} })
  Object.defineProperty(process, 'stdin', { value: readable, writable: true, configurable: true })
  readable.push(Buffer.from(data, 'utf8'))
  readable.push(null) // EOF
}

/** Capture everything written to process.stdout during `fn`. */
async function captureStdout(fn: () => Promise<void>): Promise<Buffer> {
  const chunks: Buffer[] = []
  const originalWrite = process.stdout.write.bind(process.stdout)

  // Override write to capture bytes without actually writing.
  ;(process.stdout as NodeJS.WriteStream & { write: typeof process.stdout.write }).write = (
    chunk: unknown,
    encodingOrCb?: unknown,
    cb?: unknown,
  ): boolean => {
    if (Buffer.isBuffer(chunk)) chunks.push(chunk)
    else if (typeof chunk === 'string') chunks.push(Buffer.from(chunk))
    else chunks.push(Buffer.from(chunk as Uint8Array))

    // Call the callback so the Promise in respond() resolves.
    const callback = typeof encodingOrCb === 'function' ? encodingOrCb : typeof cb === 'function' ? cb : null
    if (callback) (callback as () => void)()
    return true
  }

  try {
    await fn()
  } finally {
    ;(process.stdout as NodeJS.WriteStream & { write: typeof process.stdout.write }).write = originalWrite
  }

  return Buffer.concat(chunks)
}

// ---------------------------------------------------------------------------
// Sample payloads
// ---------------------------------------------------------------------------

const CLAUDE_CODE_PRE_TOOL_CALL = JSON.stringify({
  event: 'tool:before',
  caller: 'claude-code' as CallerKind,
  sessionId: 'session-1',
  tool: 'Bash',
  input: {
    command: 'rm -rf /tmp/test',
    description: 'Clean up temp files',
  },
})

const CLAUDE_CODE_POST_TOOL_CALL = JSON.stringify({
  event: 'tool:after',
  caller: 'claude-code' as CallerKind,
  sessionId: 'session-1',
  tool: 'Read',
  input: { file_path: '/etc/passwd' },
  output: { content: 'root:x:0:0:root:/root:/bin/bash\n', is_error: false },
})

const CURSOR_PRE_TOOL_CALL = JSON.stringify({
  event: 'tool:before',
  caller: 'cursor' as CallerKind,
  sessionId: 'session-2',
  tool: 'edit_file',
  input: { target_file: 'src/main.ts', instructions: 'refactor this function' },
})

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('helper constructors', () => {
  test('approve() returns correct shape', () => {
    expect(approve()).toEqual({ action: 'approve' })
  })

  test('block() returns correct shape with message', () => {
    expect(block('Dangerous command detected')).toEqual({
      action: 'block',
      message: 'Dangerous command detected',
    })
  })

  test('modify() returns correct shape with input', () => {
    const input = { command: 'ls /tmp' }
    expect(modify(input)).toEqual({ action: 'modify', input })
  })
})

describe('read() — Claude Code payloads', () => {
  afterEach(() => {
    // Reset cached WASM between tests so each test can inject a fresh mock.
    _setWasmInstance(null)
  })

  test('parses tool:before from Claude Code', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)
    mockStdin(CLAUDE_CODE_PRE_TOOL_CALL)

    const event = await read()

    expect(event.caller).toBe('claude-code')
    expect(event.event).toBe('tool:before')
    expect(event.tool).toBe('Bash')
  })

  test('parses tool:after from Claude Code', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)
    mockStdin(CLAUDE_CODE_POST_TOOL_CALL)

    const event = await read()

    expect(event.caller).toBe('claude-code')
    expect(event.event).toBe('tool:after')
    expect(event.tool).toBe('Read')
    expect((event.output as { is_error: boolean }).is_error).toBe(false)
  })
})

describe('read() — Cursor payloads', () => {
  afterEach(() => {
    _setWasmInstance(null)
  })

  test('parses tool:before from Cursor', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)
    mockStdin(CURSOR_PRE_TOOL_CALL)

    const event = await read()

    expect(event.caller).toBe('cursor')
    expect(event.event).toBe('tool:before')
    expect(event.tool).toBe('edit_file')
  })
})

describe('respond() — writing to stdout', () => {
  afterEach(() => {
    _setWasmInstance(null)
  })

  test('writes approve response', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)

    const approveResp = approve()
    const written = await captureStdout(() => respond(approveResp))

    const text = new TextDecoder().decode(written)
    const parsed = JSON.parse(text)
    expect(parsed.action).toBe('approve')
  })

  test('writes block response with message', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)

    const blockResp = block('rm -rf is not allowed')
    const written = await captureStdout(() => respond(blockResp))

    const text = new TextDecoder().decode(written)
    const parsed = JSON.parse(text)
    expect(parsed.action).toBe('block')
    expect(parsed.message).toBe('rm -rf is not allowed')
  })

  test('writes modify response with replacement input', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)

    const modifyResp = modify({ command: 'ls /tmp', description: 'Safe list' })
    const written = await captureStdout(() => respond(modifyResp))

    const text = new TextDecoder().decode(written)
    const parsed = JSON.parse(text)
    expect(parsed.action).toBe('modify')
    expect((parsed.input as { command: string }).command).toBe('ls /tmp')
  })
})

describe('read() + respond() round-trip', () => {
  afterEach(() => {
    _setWasmInstance(null)
  })

  test('full round-trip: Claude Code tool:before → block', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)
    mockStdin(CLAUDE_CODE_PRE_TOOL_CALL)

    const event = await read()
    expect(event.event).toBe('tool:before')

    // Simulate a hook that blocks dangerous Bash commands.
    const input = event.input as { command: string } | null
    let response: HookResponse
    if (event.tool === 'Bash' && input?.command.includes('rm -rf')) {
      response = block('Destructive command blocked by polyhook')
    } else {
      response = approve()
    }

    const written = await captureStdout(() => respond(response))
    const text = new TextDecoder().decode(written)
    const parsed = JSON.parse(text) as HookResponse
    expect(parsed.action).toBe('block')
    expect((parsed as { action: 'block'; message: string }).message).toContain('Destructive')
  })

  test('full round-trip: Cursor PreToolCall → approve', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)
    mockStdin(CURSOR_PRE_TOOL_CALL)

    const event = await read()
    const written = await captureStdout(() => respond(approve()))

    const text = new TextDecoder().decode(written)
    const parsed = JSON.parse(text) as HookResponse
    expect(parsed.action).toBe('approve')
    expect(event.caller).toBe('cursor')
  })

  test('full round-trip: Claude Code PostToolCall → modify', async () => {
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)
    mockStdin(CLAUDE_CODE_POST_TOOL_CALL)

    await read()
    const modResp = modify({ redacted: true, content: '[REDACTED]' })
    const written = await captureStdout(() => respond(modResp))

    const text = new TextDecoder().decode(written)
    const parsed = JSON.parse(text) as HookResponse & { input?: Record<string, unknown> }
    expect(parsed.action).toBe('modify')
    expect(parsed.input?.redacted).toBe(true)
  })
})

describe('mock WASM memory layout', () => {
  afterEach(() => {
    _setWasmInstance(null)
  })

  test('parse implementation uses custom normalisation logic', async () => {
    // Simulate a WASM that uppercases the tool name during parsing.
    const wasm = buildMockWasm((json) => {
      const raw = JSON.parse(json) as HookEvent
      return { ...raw, tool: raw.tool ? raw.tool.toUpperCase() : raw.tool }
    })
    _setWasmInstance(wasm)
    mockStdin(CLAUDE_CODE_PRE_TOOL_CALL)

    const event = await read()
    expect(event.tool).toBe('BASH')
  })

  test('serialize implementation can apply custom wire format', async () => {
    // Simulate a WASM that wraps the response in an envelope.
    const wasm = buildMockWasm(
      (json) => JSON.parse(json) as HookEvent,
      (resp) => JSON.stringify({ envelope: true, payload: resp }),
    )
    _setWasmInstance(wasm)

    const written = await captureStdout(() => respond(approve()))
    const text = new TextDecoder().decode(written)
    const parsed = JSON.parse(text) as { envelope: boolean; payload: HookResponse }
    expect(parsed.envelope).toBe(true)
    expect(parsed.payload.action).toBe('approve')
  })
})

// ---------------------------------------------------------------------------
// getWasm() — real loading path (fs.readFileSync error)
// ---------------------------------------------------------------------------

describe('getWasm() — WASM file not found', () => {
  beforeEach(() => {
    // Force the real WASM loading path by resetting the cached instance.
    _setWasmInstance(null)
  })

  afterEach(() => {
    vi.resetAllMocks()
    // Leave _wasm as null; subsequent test suites inject their own mock.
    _setWasmInstance(null)
  })

  test('rejects when polyhook.wasm is missing (ENOENT)', async () => {
    // Make readFileSync throw ENOENT so the getWasm() path is exercised.
    ;vi.mocked(fs.readFileSync).mockImplementation(() => {
      const err = Object.assign(new Error('ENOENT: no such file or directory'), { code: 'ENOENT' })
      throw err
    })

    // Also supply a valid stdin so the test failure comes from getWasm(), not
    // from stdin being exhausted.
    mockStdin(CLAUDE_CODE_PRE_TOOL_CALL)

    await expect(read()).rejects.toThrow('ENOENT')
  })
})

// ---------------------------------------------------------------------------
// readStdin() — error event path
// ---------------------------------------------------------------------------

describe('readStdin() — stdin error event', () => {
  afterEach(() => {
    _setWasmInstance(null)
    vi.resetAllMocks()
  })

  test('rejects when stdin emits an error event', async () => {
    // Inject a mock WASM so getWasm() succeeds and we reach readStdin().
    const wasm = buildMockWasm((json) => JSON.parse(json) as HookEvent)
    _setWasmInstance(wasm)

    // Create a Readable that will be destroyed with an error.
    const { Readable } = require('stream') as typeof import('stream')
    const errReadable = new Readable({ read() {} })
    Object.defineProperty(process, 'stdin', { value: errReadable, writable: true, configurable: true })

    // Destroy the stream asynchronously so read() has time to attach its
    // 'error' listener before the error fires.
    const stdinError = new Error('stdin pipe broken')
    setImmediate(() => errReadable.destroy(stdinError))

    await expect(read()).rejects.toThrow('stdin pipe broken')
  })
})
