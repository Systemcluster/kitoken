/* eslint-env node */

import mdx from '@astrojs/mdx'
import react from '@astrojs/react'
import { defineConfig } from 'astro/config'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
    build: {
        format: 'directory',
    },
    experimental: {
        clientPrerender: true,
        directRenderScript: true,
        serverIslands: true,
    },
    site: 'https://kitoken.dev',
    markdown: {
        gfm: true,
    },
    trailingSlash: 'never',
    devToolbar: {
        enabled: false,
    },
    integrations: [react({}), mdx({ optimize: true })],
    vite: {
        optimizeDeps: {
            exclude: ['@pandacss/dev/postcss'],
            esbuildOptions: {
                format: 'esm',
                minify: true,
                treeShaking: true,
            },
        },
        plugins: [wasm()],
    },
    output: 'static',
})
