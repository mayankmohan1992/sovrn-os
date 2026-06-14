import { defineConfig } from 'vite';
import preact from '@preact/preset-vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
  plugins: [
    preact(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.svg', 'icons/*.png'],
      manifest: {
        name: 'Sovrn Hub',
        short_name: 'Sovrn',
        start_url: '/',
        display: 'standalone',
        theme_color: '#0D9488',
        background_color: '#111827',
        icons: [
          { src: '/icons/icon-192.png', sizes: '192x192', type: 'image/png' },
          { src: '/icons/icon-512.png', sizes: '512x512', type: 'image/png' },
        ],
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}'],
      },
    }),
  ],
  server: {
    port: 54772,
    proxy: {
      '/api': 'http://127.0.0.1:54771',
      '/ws': { target: 'ws://127.0.0.1:54771', ws: true },
    },
  },
});