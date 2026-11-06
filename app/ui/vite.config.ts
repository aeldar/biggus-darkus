import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Port and host match `devUrl` in src-tauri/tauri.conf.json.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
});
