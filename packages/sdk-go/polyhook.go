// Package polyhook wraps polyhook.wasm via Wazero so that Go programs can
// parse AI-tool hook events and emit normalized responses without knowing
// which AI coding tool (Claude Code, Cursor, Windsurf, …) invoked them.
//
// Typical usage:
//
//	func main() {
//	    event, err := polyhook.Read()
//	    if err != nil {
//	        log.Fatal(err)
//	    }
//	    if event.Tool != nil && *event.Tool == "bash" {
//	        polyhook.Respond(polyhook.Block("not allowed"))
//	        return
//	    }
//	    polyhook.Respond(polyhook.Approve())
//	}
package polyhook

import (
	"context"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"sync"

	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/api"
)

// ---------------------------------------------------------------------------
// Constructor helpers
// ---------------------------------------------------------------------------

// Approve returns a HookResponse that lets the tool action proceed unchanged.
func Approve() HookResponse {
	return ApproveResponse{Action: "approve"}
}

// Block returns a HookResponse that prevents the tool action and shows
// message to the user.
func Block(message string) HookResponse {
	return BlockResponse{Action: "block", Message: message}
}

// Modify returns a HookResponse that replaces the tool's input arguments with
// the provided map.
func Modify(input map[string]interface{}) HookResponse {
	return ModifyResponse{Action: "modify", Input: input}
}

// ---------------------------------------------------------------------------
// WASM runtime — lazy, singleton
// ---------------------------------------------------------------------------

// wasmBytes holds the embedded polyhook.wasm binary. The actual embed
// directive is replaced at build time by the real file; here we use a
// package-level variable that loadWASM() populates so that tests can inject
// a custom binary.
var wasmBytes []byte

// wasmLoader may be overridden by tests to inject a custom WASM binary.
var wasmLoader func() ([]byte, error) = defaultWASMLoader

func defaultWASMLoader() ([]byte, error) {
	// 1. If the package was built with the real embed, wasmBytes is already set.
	if len(wasmBytes) > 0 {
		return wasmBytes, nil
	}

	// 2. Fall back to reading polyhook.wasm from the same directory as the
	//    running executable (useful during development / testing).
	paths := []string{
		"polyhook.wasm",
		"packages/sdk-go/polyhook.wasm",
	}
	for _, p := range paths {
		data, err := os.ReadFile(p)
		if err == nil {
			return data, nil
		}
	}
	return nil, fmt.Errorf("polyhook.wasm not found; embed it or place it next to the binary")
}

type wasmRuntime struct {
	ctx    context.Context
	rt     wazero.Runtime
	mod    api.Module
	alloc  api.Function
	dealloc api.Function
	parse   api.Function
	serialize api.Function
}

var (
	runtimeOnce sync.Once
	runtime_    *wasmRuntime
	runtimeErr  error
)

func getRuntime() (*wasmRuntime, error) {
	runtimeOnce.Do(func() {
		runtime_, runtimeErr = initRuntime()
	})
	return runtime_, runtimeErr
}

func initRuntime() (*wasmRuntime, error) {
	wasm, err := wasmLoader()
	if err != nil {
		return nil, fmt.Errorf("polyhook: load WASM: %w", err)
	}

	ctx := context.Background()
	rt := wazero.NewRuntime(ctx)

	mod, err := rt.Instantiate(ctx, wasm)
	if err != nil {
		rt.Close(ctx)
		return nil, fmt.Errorf("polyhook: instantiate WASM: %w", err)
	}

	mustExport := func(name string) (api.Function, error) {
		fn := mod.ExportedFunction(name)
		if fn == nil {
			return nil, fmt.Errorf("polyhook: WASM module missing export %q", name)
		}
		return fn, nil
	}

	allocFn, err := mustExport("alloc")
	if err != nil {
		return nil, err
	}
	deallocFn, err := mustExport("dealloc")
	if err != nil {
		return nil, err
	}
	parseFn, err := mustExport("parse")
	if err != nil {
		return nil, err
	}
	serializeFn, err := mustExport("serialize")
	if err != nil {
		return nil, err
	}

	return &wasmRuntime{
		ctx:       ctx,
		rt:        rt,
		mod:       mod,
		alloc:     allocFn,
		dealloc:   deallocFn,
		parse:     parseFn,
		serialize: serializeFn,
	}, nil
}

// ---------------------------------------------------------------------------
// Low-level WASM helpers
// ---------------------------------------------------------------------------

// wasmAlloc allocates len bytes in WASM linear memory and returns the pointer.
func (wr *wasmRuntime) wasmAlloc(length uint32) (uint32, error) {
	results, err := wr.alloc.Call(wr.ctx, uint64(length))
	if err != nil {
		return 0, fmt.Errorf("alloc(%d): %w", length, err)
	}
	return uint32(results[0]), nil
}

// wasmDealloc frees a region in WASM linear memory.
func (wr *wasmRuntime) wasmDealloc(ptr, length uint32) error {
	_, err := wr.dealloc.Call(wr.ctx, uint64(ptr), uint64(length))
	return err
}

// wasmWrite copies src into WASM linear memory at ptr.
func (wr *wasmRuntime) wasmWrite(ptr uint32, src []byte) error {
	if !wr.mod.Memory().Write(ptr, src) {
		return fmt.Errorf("Memory.Write failed at ptr=%d len=%d", ptr, len(src))
	}
	return nil
}

// wasmReadLengthPrefixed reads a length-prefixed buffer from WASM memory:
// first 4 bytes (LE i32) are the payload length, followed by the payload.
func (wr *wasmRuntime) wasmReadLengthPrefixed(ptr uint32) ([]byte, error) {
	mem := wr.mod.Memory()

	lenBytes, ok := mem.Read(ptr, 4)
	if !ok {
		return nil, fmt.Errorf("failed to read length at ptr=%d", ptr)
	}
	payloadLen := binary.LittleEndian.Uint32(lenBytes)

	payload, ok := mem.Read(ptr+4, payloadLen)
	if !ok {
		return nil, fmt.Errorf("failed to read payload at ptr=%d len=%d", ptr+4, payloadLen)
	}
	// Copy before deallocation.
	result := make([]byte, payloadLen)
	copy(result, payload)
	return result, nil
}

// wasmCall writes input bytes to a freshly allocated WASM buffer, calls fn,
// reads the length-prefixed result, and cleans up both allocations.
func (wr *wasmRuntime) wasmCall(fn api.Function, input []byte) ([]byte, error) {
	inputLen := uint32(len(input))

	// Allocate + write input.
	inPtr, err := wr.wasmAlloc(inputLen)
	if err != nil {
		return nil, err
	}
	if err := wr.wasmWrite(inPtr, input); err != nil {
		_ = wr.wasmDealloc(inPtr, inputLen)
		return nil, err
	}

	// Call export.
	results, err := fn.Call(wr.ctx, uint64(inPtr), uint64(inputLen))
	// Always free the input buffer.
	_ = wr.wasmDealloc(inPtr, inputLen)
	if err != nil {
		return nil, fmt.Errorf("WASM call: %w", err)
	}

	resultPtr := uint32(results[0])

	// Read the length-prefixed result.
	payload, err := wr.wasmReadLengthPrefixed(resultPtr)
	if err != nil {
		return nil, err
	}

	// Free the result buffer (length prefix + payload).
	_ = wr.wasmDealloc(resultPtr, 4+uint32(len(payload)))

	return payload, nil
}

// ---------------------------------------------------------------------------
// Package-level state for the last parsed caller
// ---------------------------------------------------------------------------

var (
	mu         sync.Mutex
	lastCaller CallerKind
)

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

// Read reads all bytes from stdin, passes them through polyhook.wasm's parse
// export, and returns the normalized HookEvent.
func Read() (*HookEvent, error) {
	return ReadFrom(os.Stdin)
}

// ReadFrom is like Read but sources bytes from r instead of stdin. Useful in
// tests.
func ReadFrom(r io.Reader) (*HookEvent, error) {
	raw, err := io.ReadAll(r)
	if err != nil {
		return nil, fmt.Errorf("polyhook: read stdin: %w", err)
	}

	wr, err := getRuntime()
	if err != nil {
		return nil, err
	}

	payload, err := wr.wasmCall(wr.parse, raw)
	if err != nil {
		return nil, fmt.Errorf("polyhook: parse: %w", err)
	}

	// Check for WASM-level error object.
	var errCheck struct {
		Error string `json:"error"`
	}
	if jsonErr := json.Unmarshal(payload, &errCheck); jsonErr == nil && errCheck.Error != "" {
		// polyhook.wasm still tries a best-effort parse in this case;
		// the caller falls back to "unknown". Log but do not abort.
		_ = errCheck.Error
	}

	var event HookEvent
	if err := json.Unmarshal(payload, &event); err != nil {
		return nil, fmt.Errorf("polyhook: unmarshal HookEvent: %w", err)
	}

	mu.Lock()
	lastCaller = event.Caller
	mu.Unlock()

	return &event, nil
}

// Respond serializes r through polyhook.wasm's serialize export (which
// formats the response in the format expected by the detected caller) and
// writes the result to stdout.
func Respond(r HookResponse) error {
	return RespondTo(os.Stdout, r)
}

// RespondTo is like Respond but writes to w instead of stdout. Useful in
// tests.
func RespondTo(w io.Writer, r HookResponse) error {
	responseJSON, err := json.Marshal(r)
	if err != nil {
		return fmt.Errorf("polyhook: marshal HookResponse: %w", err)
	}

	wr, err := getRuntime()
	if err != nil {
		return err
	}

	payload, err := wr.wasmCall(wr.serialize, responseJSON)
	if err != nil {
		return fmt.Errorf("polyhook: serialize: %w", err)
	}

	_, err = w.Write(payload)
	return err
}
