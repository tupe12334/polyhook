package polyhook

import "sync"

// defaultLoader is the wasmLoader value at package init time; used by
// ResetRuntime to restore state between tests.
var defaultLoader = wasmLoader

// Exported only for testing.
var (
	// ResetRuntime tears down the singleton WASM runtime and restores the
	// default wasmLoader, so the next call to getRuntime() re-initialises
	// cleanly.
	ResetRuntime = func() {
		wasmLoader = defaultLoader
		runtimeOnce = sync.Once{}
		runtime_ = nil
		runtimeErr = nil
		mu.Lock()
		lastCaller = ""
		mu.Unlock()
	}

	// SetWasmLoader replaces the package-level wasmLoader function. The
	// supplied fn is called by getRuntime() when it initialises the singleton.
	SetWasmLoader = func(fn func() ([]byte, error)) { wasmLoader = fn }

	// SetTestParser installs a Go-level mock for the WASM parse export.
	// When set, ReadFrom calls this function instead of the WASM runtime.
	SetTestParser = func(fn func([]byte) ([]byte, error)) { testParser = fn }

	// SetTestSerializer installs a Go-level mock for the WASM serialize export.
	// When set, RespondTo calls this function instead of the WASM runtime.
	SetTestSerializer = func(fn func([]byte) ([]byte, error)) { testSerializer = fn }

	// ClearTestHooks removes both mocks so subsequent calls use the real WASM runtime.
	ClearTestHooks = func() { testParser = nil; testSerializer = nil }
)
