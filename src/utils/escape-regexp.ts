const escapeRegExpRegExp = /[/\-\\^$*+?.()|[\]{}]/gu
export const escapeRegExp = (string: string) => {
    return string.replaceAll(escapeRegExpRegExp, String.raw`\$&`)
}
