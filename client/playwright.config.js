import { defineConfig, devices } from '@playwright/test'

// Chromium E2E against vite preview. API calls are mocked per-test.
// Mobile visual checks use a 400px-wide viewport (project decision G).
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  use: {
    baseURL: 'http://127.0.0.1:4173',
    colorScheme: 'dark',
    reducedMotion: 'reduce',
  },
  webServer: {
    command: 'bun run build && bun run preview --port 4173 --strictPort --host 127.0.0.1',
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
  projects: [
    {
      name: 'mobile-400',
      testMatch: /chrome-mobile\.spec\.js$/,
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 400, height: 800 },
        isMobile: true,
        hasTouch: true,
      },
    },
    {
      name: 'desktop-1280',
      testMatch: /chrome-desktop\.spec\.js$/,
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1280, height: 800 },
      },
    },
  ],
})
