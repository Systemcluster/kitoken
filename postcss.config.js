/* eslint-env node */

import panda from '@pandacss/dev/postcss'
import presetenv from 'postcss-preset-env'
// @ts-ignore
import removedecl from 'postcss-remove-declaration'

/** @type {import('postcss-load-config').Config} */
const config = {
    plugins: [
        panda({}),
        presetenv({
            stage: 2,
            features: {},
            autoprefixer: {
                flexbox: 'no-2009',
                grid: 'autoplace',
            },
        }),
        removedecl({
            remove: {
                ':root': '--made-with-panda',
            },
        }),
    ],
}

export default config
