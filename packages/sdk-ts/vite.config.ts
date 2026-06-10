import { defineConfig } from "vite";
import dts from "vite-plugin-dts";
import type { Plugin } from "vite";

// Rolldown (Vite 8) replaces `import.meta.url` with `{}.url` in CJS output.
// Patch it to the correct Node.js CJS equivalent.
function fixCjsImportMetaUrl(): Plugin {
  return {
    name: "fix-cjs-import-meta-url",
    generateBundle(options, bundle) {
      if (options.format !== "cjs") return;
      for (const chunk of Object.values(bundle)) {
        if (chunk.type === "chunk") {
          chunk.code = chunk.code.replace(
            /\{\}\.url/g,
            "require('url').pathToFileURL(__filename).href",
          );
        }
      }
    },
  };
}

export default defineConfig({
  build: {
    lib: {
      entry: "./src/index.ts",
      formats: ["es", "cjs"],
      fileName: (format) => (format === "es" ? "index.js" : "index.cjs"),
    },
    rollupOptions: {
      external: [/^node:/, "fs", "path", "url"],
    },
  },
  plugins: [dts({ insertTypesEntry: true }), fixCjsImportMetaUrl()],
});
