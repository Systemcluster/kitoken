/* eslint-env node */

import { astro, base, biome, jest, react, typescript, valtio } from '@systemcluster/eslint-config'

base.rules ? (base.rules['unicorn/filename-case'] = 'off') : 0
valtio.rules ? (valtio.rules['valtio/state-snapshot-rule'] = 'off') : 0

export default [base, typescript, react, jest, valtio, biome, astro]
