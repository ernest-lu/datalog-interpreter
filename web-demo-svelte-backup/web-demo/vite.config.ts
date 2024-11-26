import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import wasmPack from 'vite-plugin-wasm-pack';

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte(), wasmPack('./datalog_wasm')],
})
