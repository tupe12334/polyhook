# Polyhook — top-level Makefile
#
# All SDK types (CallerKind, HookEvent, HookResponse) are generated from
# schema.json.  Run `make schema` (the default) to regenerate every SDK, or
# run a language-specific target to regenerate only one.
#
# Prerequisites (install once):
#   TypeScript  : npm install -g json-schema-to-typescript
#   Go          : go install github.com/omissis/go-jsonschema/cmd/gojsonschema@latest
#   Python      : pip install datamodel-code-generator
#   .NET        : dotnet tool install --global NJsonSchema.CodeGeneration.CLI  (or use NSwag)
#   Rust        : types are hand-maintained in core/src/types.rs
#                 (marked "generated from schema.json"); build.rs triggers revalidation
#   wasm32 target : rustup target add wasm32-unknown-unknown
#   cspell      : npm install -g cspell

SCHEMA        := schema.json
TS_OUT        := packages/sdk-ts/src/generated/types.ts
GO_OUT        := packages/sdk-go/generated_types.go
PYTHON_OUT    := packages/sdk-python/src/polyhook/generated_models.py
DOTNET_OUT    := packages/sdk-dotnet/GeneratedTypes.cs
WASM_OUT      := polyhook.wasm

.PHONY: all schema schema/rust schema/ts schema/go schema/dotnet schema/python \
        wasm test spell install-hooks readme help

# ── Default ──────────────────────────────────────────────────────────────────
all: schema

# ── schema (all) ─────────────────────────────────────────────────────────────
## schema: Regenerate types for every SDK from schema.json (default target)
schema: schema/ts schema/go schema/python schema/dotnet schema/rust
	@echo "All SDK types regenerated from $(SCHEMA)."

# ── schema/rust ───────────────────────────────────────────────────────────────
## schema/rust: Remind how Rust types are kept in sync with schema.json
schema/rust:
	@echo "──────────────────────────────────────────────────────"
	@echo "Rust types are maintained in:"
	@echo "  core/src/types.rs"
	@echo ""
	@echo "They are hand-written but clearly marked with a comment:"
	@echo "  // Source of truth: schema.json — keep in sync manually."
	@echo ""
	@echo "build.rs emits:"
	@echo "  cargo:rerun-if-changed=../schema.json"
	@echo "so Cargo invalidates the build whenever schema.json changes."
	@echo ""
	@echo "Run 'cargo build -p polyhook-core' to recompile."
	@echo "──────────────────────────────────────────────────────"

# ── schema/ts ─────────────────────────────────────────────────────────────────
## schema/ts: Generate TypeScript types from schema.json using json-schema-to-typescript
schema/ts: $(SCHEMA)
	@echo "Generating TypeScript types → $(TS_OUT)"
	@mkdir -p $(dir $(TS_OUT))
	npx --yes json-schema-to-typescript \
	    $(SCHEMA) \
	    --unreachableDefinitions \
	    --no-additionalProperties \
	    > $(TS_OUT)
	@# Fix dedup artifact: tool emits CallerKind1 for the $ref inline use; collapse to CallerKind.
	sed -i.bak '/^export type CallerKind1 =/,/^$$/d' $(TS_OUT) && rm -f $(TS_OUT).bak
	sed -i.bak 's/CallerKind1/CallerKind/g' $(TS_OUT) && rm -f $(TS_OUT).bak
	@echo "Done: $(TS_OUT)"

# ── schema/go ─────────────────────────────────────────────────────────────────
## schema/go: Generate Go types from schema.json using go-jsonschema
schema/go: $(SCHEMA)
	@echo "Generating Go types → $(GO_OUT)"
	@mkdir -p $(dir $(GO_OUT))
	gojsonschema \
	    --schema-package https://polyhook.dev/schema.json=polyhook \
	    --output $(GO_OUT) \
	    $(SCHEMA)
	@echo "Done: $(GO_OUT)"

# ── schema/dotnet ─────────────────────────────────────────────────────────────
## schema/dotnet: Generate .NET C# types from schema.json using NJsonSchema
schema/dotnet: $(SCHEMA)
	@echo "Generating .NET C# types → $(DOTNET_OUT)"
	@mkdir -p $(dir $(DOTNET_OUT))
	@# NJsonSchema CLI: install with `dotnet tool install --global NJsonSchema.CodeGeneration.CLI`
	@# or NSwag: install with `dotnet tool install --global NSwag.ConsoleCore`
	@if command -v njsonschema >/dev/null 2>&1; then \
	    njsonschema \
	        generate-types \
	        --input $(SCHEMA) \
	        --output $(DOTNET_OUT) \
	        --namespace Polyhook \
	        --class-name PolyhookTypes; \
	elif command -v nswag >/dev/null 2>&1; then \
	    nswag jsonschema2csclient \
	        /input:$(SCHEMA) \
	        /output:$(DOTNET_OUT) \
	        /namespace:Polyhook; \
	else \
	    echo "ERROR: Neither 'njsonschema' nor 'nswag' found."; \
	    echo "Install one of:"; \
	    echo "  dotnet tool install --global NJsonSchema.CodeGeneration.CLI"; \
	    echo "  dotnet tool install --global NSwag.ConsoleCore"; \
	    exit 1; \
	fi
	@echo "Done: $(DOTNET_OUT)"

# ── schema/python ─────────────────────────────────────────────────────────────
## schema/python: Generate Python Pydantic models from schema.json using datamodel-code-generator
schema/python: $(SCHEMA)
	@echo "Generating Python types → $(PYTHON_OUT)"
	@mkdir -p $(dir $(PYTHON_OUT))
	datamodel-codegen \
	    --input $(SCHEMA) \
	    --input-file-type jsonschema \
	    --output $(PYTHON_OUT) \
	    --output-model-type pydantic_v2.BaseModel \
	    --target-python-version 3.10 \
	    --use-standard-collections \
	    --use-union-operator \
	    --field-constraints \
	    --wrap-string-literal \
	    --custom-file-header "# DO NOT EDIT — generated from schema.json by \`make schema/python\`."
	@echo "Done: $(PYTHON_OUT)"

# ── wasm ──────────────────────────────────────────────────────────────────────
## wasm: Build polyhook.wasm via cargo (wasm32-unknown-unknown, release)
wasm:
	@echo "Building polyhook.wasm…"
	cargo build --release --target wasm32-unknown-unknown -p polyhook-core
	cp target/wasm32-unknown-unknown/release/polyhook_core.wasm $(WASM_OUT)
	cp $(WASM_OUT) packages/sdk-ts/polyhook.wasm
	cp $(WASM_OUT) packages/sdk-go/polyhook.wasm
	cp $(WASM_OUT) packages/sdk-python/src/polyhook/polyhook.wasm
	cp $(WASM_OUT) packages/sdk-dotnet/polyhook.wasm
	@echo "Done: $(WASM_OUT) → all SDK directories"

# ── test ──────────────────────────────────────────────────────────────────────
## test: Run the full test suite across all SDKs
test:
	@echo "── Rust ────────────────────────────────────────────────"
	cargo test
	@echo "── TypeScript ──────────────────────────────────────────"
	npm test -w sdk-ts
	@echo "── Go ──────────────────────────────────────────────────"
	cd packages/sdk-go && go test ./...
	@echo "── .NET ────────────────────────────────────────────────"
	dotnet test packages/sdk-dotnet
	@echo "── Python ──────────────────────────────────────────────"
	python -m pytest packages/sdk-python
	@echo "All tests passed."

# ── spell ─────────────────────────────────────────────────────────────────────
## spell: Run cspell spell check across the repository
spell:
	cspell "**" --no-progress

# ── readme ────────────────────────────────────────────────────────────────────
## readme: Generate README.md for every SDK package from scripts/gen-readmes.sh
readme:
	@echo "Generating package READMEs…"
	@bash scripts/gen-readmes.sh

# ── install-hooks ─────────────────────────────────────────────────────────────
## install-hooks: Configure git to use the committed githooks/ directory
install-hooks:
	git config core.hooksPath githooks
	@echo "Git hooks installed from githooks/."

# ── help ──────────────────────────────────────────────────────────────────────
## help: Show this help message
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/^## /  /'
	@echo ""
	@echo "Prerequisites:"
	@echo "  TypeScript : npm install -g json-schema-to-typescript"
	@echo "  Go         : go install github.com/omissis/go-jsonschema/cmd/gojsonschema@latest"
	@echo "  Python     : pip install datamodel-code-generator"
	@echo "  .NET       : dotnet tool install --global NJsonSchema.CodeGeneration.CLI"
	@echo "  wasm32 target : rustup target add wasm32-unknown-unknown"

# ── Coverage (100% required, generated code excluded) ─────────────────────────

## test-coverage: Run all SDKs with 100% coverage enforcement
test-coverage: coverage/rust coverage/ts coverage/go coverage/python coverage/dotnet
	@echo "All coverage checks passed."

## coverage/rust: Rust (core + sdk-rust) coverage via cargo-llvm-cov
coverage/rust:
	cargo llvm-cov --workspace \
		--ignore-filename-regex 'types\.rs$$|wasm\.rs$$' \
		--fail-under-lines 100 \
		--lcov --output-path target/lcov-rust.info \
		-- --test-threads 1

## coverage/ts: TypeScript SDK coverage via Vitest
coverage/ts:
	cd packages/sdk-ts && pnpm test:coverage

## coverage/go: Go SDK coverage (exclude generated_types.go)
## Builds polyhook.wasm first so WASM-path tests run (threshold 85% since
## wazero memory-error paths require a buggy WASM to trigger).
coverage/go:
	cargo build -p polyhook-core --target wasm32-unknown-unknown --release --quiet
	cp target/wasm32-unknown-unknown/release/polyhook_core.wasm packages/sdk-go/polyhook.wasm
	cd packages/sdk-go && \
	  go test -coverprofile=coverage.out \
	     -coverpkg=$$(go list ./... | grep -v '^$$') \
	     ./... && \
	  go tool cover -func=coverage.out | \
	    grep -v 'generated_types\.go' | \
	    awk '/total:/{pct=$$3+0; if (pct < 85) { print "Go coverage: "$$3" (need ≥85%)"; exit 1 }}'

## coverage/python: Python SDK coverage via pytest-cov
coverage/python:
	cd packages/sdk-python && \
	  uv run --extra dev pytest --cov=polyhook --cov-report=term-missing \
	    --cov-fail-under=100 \
	    --cov-config=.coveragerc

## coverage/dotnet: .NET SDK coverage via coverlet
## Builds polyhook.wasm first (required as embedded resource in Polyhook.Sdk).
coverage/dotnet:
	cargo build -p polyhook-core --target wasm32-unknown-unknown --release --quiet
	cp target/wasm32-unknown-unknown/release/polyhook_core.wasm packages/sdk-dotnet/polyhook.wasm
	cd packages/sdk-dotnet && \
	  dotnet test PolyhookTests/ \
	    /p:CollectCoverage=true \
	    /p:CoverletOutputFormat=opencover \
	    /p:Exclude="[*]*.GeneratedTypes" \
	    /p:ThresholdType=line \
	    /p:Threshold=100

.PHONY: test-coverage coverage/rust coverage/ts coverage/go coverage/python coverage/dotnet
