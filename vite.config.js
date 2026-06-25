import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

const NODE_BUILTINS = ["child_process", "fs", "module", "path", "url"];

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    sveltekit(),
    {
      name: "file2md-stubs",
      enforce: "pre",
      resolveId(id, importer) {
        if (importer && importer.includes("@covoyage/file2md")) {
          if (id === "fs/promises") {
            return "\0file2md-node-stub:fs/promises";
          }
          if (NODE_BUILTINS.includes(id)) {
            return "\0file2md-node-stub:" + id;
          }
          if (
            id.includes("transcribe-audio-node") ||
            id.includes("pdf-pdftotext-node") ||
            id.includes("resolve-exiftool-node") ||
            id.includes("plugin-discovery-node") ||
            id.includes("pdf-plumber-node") ||
            id.includes("import-node-util") ||
            id.includes("xlsx-pandas-node")
          ) {
            return "\0file2md-stub:" + id.split("/").pop();
          }
        }
        return null;
      },
      load(id) {
        if (id === "\0file2md-node-stub:child_process") {
          return `export function spawn() {}; export function exec() {};`;
        }
        if (id === "\0file2md-node-stub:fs") {
          return `export function access() {}; export function readFileSync() { return ''; };`;
        }
        if (id === "\0file2md-node-stub:fs/promises") {
          return `export function access() { return Promise.resolve(); };`;
        }
        if (id === "\0file2md-node-stub:module") {
          return `export function createRequire() { return () => { throw new Error('Not available in browser'); }; };`;
        }
        if (id === "\0file2md-node-stub:path") {
          return `export function dirname() { return '.'; }; export function join(...p) { return p.join('/'); };`;
        }
        if (id === "\0file2md-node-stub:url") {
          return `export function pathToFileURL() { return new URL('file:///'); }; export function fileURLToPath() { return '/'; };`;
        }
        if (id && id.startsWith("\0file2md-stub:")) {
          return "export {};";
        }
        return null;
      },
    },
  ],

  optimizeDeps: {
    include: ["@covoyage/file2md", "xlsx", "pdfjs-dist"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
