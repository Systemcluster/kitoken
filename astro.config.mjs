/* eslint-env node */

import mdx from '@astrojs/mdx'
import react from '@astrojs/react'
import { defineConfig } from 'astro/config'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
    build: {},
    experimental: {
        clientPrerender: true,
        directRenderScript: false,
        serverIslands: false,
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
        },
        plugins: [wasm()],
    },
    output: 'static',
})
