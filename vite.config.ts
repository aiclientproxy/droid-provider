import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import fs from 'fs';

const isCI = process.env.CI === 'true' || process.env.GITHUB_ACTIONS === 'true';
const proxycastComponentsPath = path.resolve(__dirname, '../proxycast/src/lib/plugin-components');
const hasLocalComponents = fs.existsSync(proxycastComponentsPath);

export default defineConfig({
  plugins: [react()],
  define: {
    'process.env.NODE_ENV': JSON.stringify('production'),
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      ...(hasLocalComponents && !isCI ? {
        '@proxycast/plugin-components': proxycastComponentsPath,
      } : {}),
    },
  },
  build: {
    outDir: 'plugin/dist',
    lib: {
      entry: path.resolve(__dirname, 'src/index.tsx'),
      name: 'DroidProviderUI',
      formats: ['iife'],
      fileName: () => 'index.js',
    },
    rollupOptions: {
      external: ['react', 'react-dom', '@proxycast/plugin-components'],
      output: {
        globals: {
          'react': 'React',
          'react-dom': 'ReactDOM',
          '@proxycast/plugin-components': 'ProxyCastPluginComponents',
        },
        name: 'DroidProviderPlugin',
        exports: 'named',
      },
    },
    cssCodeSplit: false,
  },
});
