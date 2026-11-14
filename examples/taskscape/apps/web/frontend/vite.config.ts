import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// In development the API runs on its own port (manage.py runserver 8000); the dev server forwards
// /api and /media to it, so the browser sees one origin, as it does in production.
const api = process.env.API_URL ?? "http://127.0.0.1:8000";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: { port: 5173, proxy: { "/api": api, "/media": api } },
  preview: { port: 5173, proxy: { "/api": api, "/media": api } },
});
