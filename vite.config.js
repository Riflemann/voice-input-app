import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],
  build: {
    outDir: 'dist',  // Убедитесь, что указано 'dist'
    emptyOutDir: true
  },

  // Опции Vite, настроенные для разработки с Tauri (применяются в `tauri dev` и `tauri build`)
  //
  // 1. предотвращаем, чтобы Vite скрывал ошибки Rust
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  // 2. Tauri ожидает фиксированный порт — падать, если порт недоступен
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. сказать Vite игнорировать папку `src-tauri` при watch
      ignored: ["**/src-tauri/**"],
    },
  },
}));
