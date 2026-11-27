import { svelte } from '@sveltejs/vite-plugin-svelte'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'
import { svelteTesting } from '@testing-library/svelte/vite'
import { defineConfig } from 'vite'
import path from 'path'

export default defineConfig({
  plugins: [
    svelte({ preprocess: vitePreprocess() }),
    // The tests of the page clean up after themselves: a test of a pure function loads none of
    // the testing library.
    svelteTesting({ autoCleanup: false }),
  ],
  resolve: {
    alias: {
      $lib: path.resolve('./src/lib'),
    },
  },
  base: './',
  build: {
    outDir: 'build',
  },
  server: {
    host: '0.0.0.0',
    port: 5173,
    proxy: {
      '/api': 'http://localhost:3000',
    },
  },
  test: {
    include: ['src/**/*.test.{js,ts}'],
    // A test of the page says `@vitest-environment jsdom`. The components' styles are in its
    // document: getComputedStyle tells what they set. jsdom applies no @media rule, so the page
    // has the styles of a window 800px or wider.
    css: true,
  },
})
