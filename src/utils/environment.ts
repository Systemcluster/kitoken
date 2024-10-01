declare global {
    interface Document {
        documentMode?: unknown
    }
    interface Window {
        MSStream?: unknown
    }
}

export const HAS_DOM = (() => {
    // eslint-disable-next-line @typescript-eslint/no-deprecated
    return typeof window !== 'undefined' && typeof window.document.createElement !== 'undefined'
})()

export const HAS_BEFORE_INPUT = (() => {
    if (!HAS_DOM) return false
    if ('InputEvent' in window && !('documentMode' in document)) {
        return 'getTargetRanges' in new window.InputEvent('input')
    }
    return false
})()

export const HAS_SHARED_WORKER = (() => {
    return typeof SharedWorker !== 'undefined'
})()

export const IS_IE = (() => {
    if (!HAS_DOM) return false
    return 'documentMode' in document
})()

export const IS_CHROME = (() => {
    if (!HAS_DOM) return false
    if (typeof navigator !== 'undefined' && 'userAgent' in navigator) {
        return /^(?=.*chrome).*/iu.test(navigator.userAgent)
    }
    return false
})()

export const IS_FIREFOX = (() => {
    if (!HAS_DOM) return false
    if (typeof navigator !== 'undefined' && 'userAgent' in navigator) {
        return /^(?!.*seamonkey)(?=.*firefox).*/iu.test(navigator.userAgent)
    }
    return false
})()

export const IS_SAFARI = (() => {
    if (!HAS_DOM) return false
    if (typeof navigator !== 'undefined' && 'userAgent' in navigator) {
        return /Version\/[\d.]+.*Safari/u.test(navigator.userAgent)
    }
    return false
})()

export const IS_IOS = (() => {
    if (!HAS_DOM) return false
    if (window.MSStream) return false
    if (typeof navigator !== 'undefined' && 'userAgent' in navigator) {
        return /iphone|ipad|ipod/iu.test(navigator.userAgent)
    }
    return false
})()

export const IS_APPLE_WEBKIT = (() => {
    if (!HAS_DOM) return false
    if (IS_CHROME) return false
    if (typeof navigator !== 'undefined' && 'userAgent' in navigator) {
        return /AppleWebKit\/[\d.]+/u.test(navigator.userAgent)
    }
    return false
})()

export const IS_APPLE = (() => {
    if (!HAS_DOM) return false
    if (typeof navigator !== 'undefined' && navigator.userAgentData?.platform !== undefined) {
        return /mac|ipod|iphone|ipad/iu.test(navigator.userAgentData.platform)
    }
    if (typeof navigator !== 'undefined' && 'platform' in navigator) {
        // eslint-disable-next-line @typescript-eslint/no-deprecated
        return /mac|ipod|iphone|ipad/iu.test(navigator.platform)
    }
    return false
})()

export const IS_MOBILE = (() => {
    if (!HAS_DOM) return false
    // test if user agent data advertises itself as mobile
    if (typeof navigator !== 'undefined' && navigator.userAgentData?.mobile !== undefined) {
        return navigator.userAgentData.mobile
    }
    // test if window has orientation property
    if (typeof window !== 'undefined' && 'orientation' in window) {
        return true
    }
    // test if user agent contains mobi
    if (typeof navigator !== 'undefined' && 'userAgent' in navigator) {
        return /mobi/iu.test(navigator.userAgent)
    }
    return false
})()

export const IS_TOUCH_DEVICE = (() => {
    if (!HAS_DOM) return false
    if (typeof window !== 'undefined') {
        return window.matchMedia('(hover: none)').matches
    }
    return false
})()

export const IS_DEV_MODE = (() => {
    if (typeof import.meta !== 'undefined' && import.meta.env.MODE === 'development') {
        return true
    }
    if (typeof process !== 'undefined' && process.env.NODE_ENV === 'development') {
        return true
    }
    return false
})()
