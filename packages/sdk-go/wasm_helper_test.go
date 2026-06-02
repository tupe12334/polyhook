package polyhook_test

// wasm_helper_test.go provides a pure-Go passthrough shim used by all tests
// that exercise the WASM glue code (ReadFrom, RespondTo, …).
//
// The shim bypasses wazero entirely via the testParser / testSerializer hooks
// added to polyhook.go for testing:
//
//   - parse: returns the input bytes unchanged (the test caller is responsible
//     for supplying pre-normalised HookEvent JSON).
//   - serialize: returns the response JSON unchanged.
//
// A size check in parse rejects inputs larger than two pages of WASM memory
// (131 072 bytes) to cover the memory-overflow error path that the real WASM
// module would trigger.

import (
	"fmt"

	polyhook "github.com/polyhook/polyhook-go"
)

const wasmMemoryLimit = 131072 // 2 pages × 65536 bytes/page

// passthroughParse returns the input bytes unchanged, simulating a WASM parse
// export that already received pre-normalised JSON.
func passthroughParse(input []byte) ([]byte, error) {
	if len(input) > wasmMemoryLimit {
		return nil, fmt.Errorf("input length %d exceeds WASM memory limit %d", len(input), wasmMemoryLimit)
	}
	return input, nil
}

// passthroughSerialize returns the response JSON unchanged.
func passthroughSerialize(input []byte) ([]byte, error) {
	return input, nil
}

// usePassthroughWASM installs the pure-Go passthrough shims and resets the
// WASM runtime singleton so it will not be initialised during the test.
func usePassthroughWASM() {
	polyhook.SetTestParser(passthroughParse)
	polyhook.SetTestSerializer(passthroughSerialize)
}

// resetRuntime tears down the Go shims and the runtime singleton so
// subsequent tests start fresh.
func resetRuntime() {
	polyhook.ClearTestHooks()
	polyhook.ResetRuntime()
}
