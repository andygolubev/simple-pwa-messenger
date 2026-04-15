import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: "dist",
    target: "es2022",
  },
  server: {
    port: 3000,
  },
  test: {
    environment: "jsdom",
    globals: true,
  },
});
