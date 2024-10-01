/* eslint-disable */
export type Token = "colors.transparent" | "colors.background.primary" | "colors.background.secondary" | "colors.background.backdrop" | "colors.background.overlay" | "colors.background.muted" | "colors.background.inverse" | "colors.text.primary" | "colors.text.secondary" | "colors.text.tertiary" | "colors.text.muted" | "colors.border.primary" | "colors.border.active" | "colors.shadow.primary" | "colors.shadow.muted" | "colors.brand.primary.blue" | "colors.brand.primary.green" | "colors.brand.primary.yellow" | "colors.brand.primary.red" | "colors.brand.primary.teal" | "colors.brand.primary.purple" | "colors.brand.primary.orange" | "colors.brand.primary.pink" | "colors.brand.primary.gold" | "colors.brand.secondary.blue" | "colors.brand.secondary.green" | "colors.brand.secondary.yellow" | "colors.brand.secondary.red" | "colors.brand.secondary.teal" | "colors.brand.secondary.purple" | "colors.brand.secondary.orange" | "colors.brand.secondary.pink" | "colors.brand.secondary.gold" | "fonts.main" | "fonts.head" | "fonts.text" | "fonts.mono" | "fontWeights.main" | "fontWeights.head" | "fontWeights.thin" | "fontWeights.light" | "fontWeights.regular" | "fontWeights.medium" | "fontWeights.bold" | "fontWeights.bolder" | "fontWeights.black" | "fontWeights.massive" | "spacing.xs" | "spacing.sm" | "spacing.md" | "spacing.lg" | "spacing.xl" | "spacing.x2" | "spacing.x3" | "spacing.x4" | "spacing.x5" | "spacing.x6" | "spacing.x7" | "spacing.x8" | "sizes.xs" | "sizes.sm" | "sizes.md" | "sizes.lg" | "sizes.xl" | "sizes.x2" | "sizes.x3" | "sizes.x4" | "sizes.x5" | "sizes.x6" | "sizes.x7" | "sizes.x8" | "sizes.breakpoint-sm" | "sizes.breakpoint-md" | "sizes.breakpoint-lg" | "fontSizes.xs" | "fontSizes.sm" | "fontSizes.md" | "fontSizes.lg" | "fontSizes.xl" | "fontSizes.x2" | "fontSizes.x3" | "fontSizes.x4" | "fontSizes.x5" | "fontSizes.x6" | "lineHeights.xs" | "lineHeights.sm" | "lineHeights.md" | "lineHeights.lg" | "lineHeights.xl" | "lineHeights.x2" | "lineHeights.x3" | "lineHeights.x4" | "lineHeights.x5" | "lineHeights.x6" | "letterSpacings.xs" | "letterSpacings.sm" | "letterSpacings.md" | "letterSpacings.lg" | "letterSpacings.xl" | "letterSpacings.x2" | "letterSpacings.x3" | "letterSpacings.x4" | "letterSpacings.x5" | "letterSpacings.x6" | "radii.full" | "radii.xs" | "radii.sm" | "radii.md" | "radii.lg" | "radii.xl" | "borderWidths.none" | "borderWidths.xs" | "borderWidths.sm" | "borderWidths.md" | "borderWidths.lg" | "borderWidths.xl" | "shadows.none" | "shadows.xs" | "shadows.sm" | "shadows.md" | "shadows.lg" | "shadows.xl" | "durations.none" | "durations.fast" | "durations.normal" | "durations.medium" | "durations.slow" | "durations.slower" | "durations.slowest" | "easings.none" | "easings.linear" | "easings.ease" | "easings.easeSmooth" | "easings.easeIn" | "easings.easeInQuad" | "easings.easeInCubic" | "easings.easeInQuart" | "easings.easeInQuint" | "easings.easeInSine" | "easings.easeInExpo" | "easings.easeInCirc" | "easings.easeInBack" | "easings.easeOut" | "easings.easeOutQuad" | "easings.easeOutCubic" | "easings.easeOutQuart" | "easings.easeOutQuint" | "easings.easeOutSine" | "easings.easeOutExpo" | "easings.easeOutCirc" | "easings.easeOutBack" | "easings.easeInOut" | "easings.easeInOutQuad" | "easings.easeInOutCubic" | "easings.easeInOutQuart" | "easings.easeInOutQuint" | "easings.easeInOutSine" | "easings.easeInOutExpo" | "easings.easeInOutCirc" | "easings.easeInOutBack" | "animations.spin" | "animations.fadeIn" | "animations.scrollX" | "animations.scrollY" | "animations.flickerX" | "animations.flickerY" | "animations.moveDown" | "animations.moveUp" | "animations.moveLeft" | "animations.moveRight" | "animations.scaleIn" | "animations.scaleOut" | "breakpoints.sm" | "breakpoints.md" | "breakpoints.lg" | "spacing.-xs" | "spacing.-sm" | "spacing.-md" | "spacing.-lg" | "spacing.-xl" | "spacing.-x2" | "spacing.-x3" | "spacing.-x4" | "spacing.-x5" | "spacing.-x6" | "spacing.-x7" | "spacing.-x8" | "colors.colorPalette" | "colors.colorPalette.primary" | "colors.colorPalette.secondary" | "colors.colorPalette.backdrop" | "colors.colorPalette.overlay" | "colors.colorPalette.muted" | "colors.colorPalette.inverse" | "colors.colorPalette.tertiary" | "colors.colorPalette.active" | "colors.colorPalette.primary.blue" | "colors.colorPalette.blue" | "colors.colorPalette.primary.green" | "colors.colorPalette.green" | "colors.colorPalette.primary.yellow" | "colors.colorPalette.yellow" | "colors.colorPalette.primary.red" | "colors.colorPalette.red" | "colors.colorPalette.primary.teal" | "colors.colorPalette.teal" | "colors.colorPalette.primary.purple" | "colors.colorPalette.purple" | "colors.colorPalette.primary.orange" | "colors.colorPalette.orange" | "colors.colorPalette.primary.pink" | "colors.colorPalette.pink" | "colors.colorPalette.primary.gold" | "colors.colorPalette.gold" | "colors.colorPalette.secondary.blue" | "colors.colorPalette.secondary.green" | "colors.colorPalette.secondary.yellow" | "colors.colorPalette.secondary.red" | "colors.colorPalette.secondary.teal" | "colors.colorPalette.secondary.purple" | "colors.colorPalette.secondary.orange" | "colors.colorPalette.secondary.pink" | "colors.colorPalette.secondary.gold"

export type ColorPalette = "transparent" | "background" | "text" | "border" | "shadow" | "brand" | "brand.primary" | "brand.secondary"

export type ColorToken = "transparent" | "background.primary" | "background.secondary" | "background.backdrop" | "background.overlay" | "background.muted" | "background.inverse" | "text.primary" | "text.secondary" | "text.tertiary" | "text.muted" | "border.primary" | "border.active" | "shadow.primary" | "shadow.muted" | "brand.primary.blue" | "brand.primary.green" | "brand.primary.yellow" | "brand.primary.red" | "brand.primary.teal" | "brand.primary.purple" | "brand.primary.orange" | "brand.primary.pink" | "brand.primary.gold" | "brand.secondary.blue" | "brand.secondary.green" | "brand.secondary.yellow" | "brand.secondary.red" | "brand.secondary.teal" | "brand.secondary.purple" | "brand.secondary.orange" | "brand.secondary.pink" | "brand.secondary.gold" | "colorPalette" | "colorPalette.primary" | "colorPalette.secondary" | "colorPalette.backdrop" | "colorPalette.overlay" | "colorPalette.muted" | "colorPalette.inverse" | "colorPalette.tertiary" | "colorPalette.active" | "colorPalette.primary.blue" | "colorPalette.blue" | "colorPalette.primary.green" | "colorPalette.green" | "colorPalette.primary.yellow" | "colorPalette.yellow" | "colorPalette.primary.red" | "colorPalette.red" | "colorPalette.primary.teal" | "colorPalette.teal" | "colorPalette.primary.purple" | "colorPalette.purple" | "colorPalette.primary.orange" | "colorPalette.orange" | "colorPalette.primary.pink" | "colorPalette.pink" | "colorPalette.primary.gold" | "colorPalette.gold" | "colorPalette.secondary.blue" | "colorPalette.secondary.green" | "colorPalette.secondary.yellow" | "colorPalette.secondary.red" | "colorPalette.secondary.teal" | "colorPalette.secondary.purple" | "colorPalette.secondary.orange" | "colorPalette.secondary.pink" | "colorPalette.secondary.gold"

export type FontToken = "main" | "head" | "text" | "mono"

export type FontWeightToken = "main" | "head" | "thin" | "light" | "regular" | "medium" | "bold" | "bolder" | "black" | "massive"

export type SpacingToken = "xs" | "sm" | "md" | "lg" | "xl" | "x2" | "x3" | "x4" | "x5" | "x6" | "x7" | "x8" | "-xs" | "-sm" | "-md" | "-lg" | "-xl" | "-x2" | "-x3" | "-x4" | "-x5" | "-x6" | "-x7" | "-x8"

export type SizeToken = "xs" | "sm" | "md" | "lg" | "xl" | "x2" | "x3" | "x4" | "x5" | "x6" | "x7" | "x8" | "breakpoint-sm" | "breakpoint-md" | "breakpoint-lg"

export type FontSizeToken = "xs" | "sm" | "md" | "lg" | "xl" | "x2" | "x3" | "x4" | "x5" | "x6"

export type LineHeightToken = "xs" | "sm" | "md" | "lg" | "xl" | "x2" | "x3" | "x4" | "x5" | "x6"

export type LetterSpacingToken = "xs" | "sm" | "md" | "lg" | "xl" | "x2" | "x3" | "x4" | "x5" | "x6"

export type RadiusToken = "full" | "xs" | "sm" | "md" | "lg" | "xl"

export type BorderWidthToken = "none" | "xs" | "sm" | "md" | "lg" | "xl"

export type ShadowToken = "none" | "xs" | "sm" | "md" | "lg" | "xl"

export type DurationToken = "none" | "fast" | "normal" | "medium" | "slow" | "slower" | "slowest"

export type EasingToken = "none" | "linear" | "ease" | "easeSmooth" | "easeIn" | "easeInQuad" | "easeInCubic" | "easeInQuart" | "easeInQuint" | "easeInSine" | "easeInExpo" | "easeInCirc" | "easeInBack" | "easeOut" | "easeOutQuad" | "easeOutCubic" | "easeOutQuart" | "easeOutQuint" | "easeOutSine" | "easeOutExpo" | "easeOutCirc" | "easeOutBack" | "easeInOut" | "easeInOutQuad" | "easeInOutCubic" | "easeInOutQuart" | "easeInOutQuint" | "easeInOutSine" | "easeInOutExpo" | "easeInOutCirc" | "easeInOutBack"

export type AnimationToken = "spin" | "fadeIn" | "scrollX" | "scrollY" | "flickerX" | "flickerY" | "moveDown" | "moveUp" | "moveLeft" | "moveRight" | "scaleIn" | "scaleOut"

export type BreakpointToken = "sm" | "md" | "lg"

export type Tokens = {
		colors: ColorToken
		fonts: FontToken
		fontWeights: FontWeightToken
		spacing: SpacingToken
		sizes: SizeToken
		fontSizes: FontSizeToken
		lineHeights: LineHeightToken
		letterSpacings: LetterSpacingToken
		radii: RadiusToken
		borderWidths: BorderWidthToken
		shadows: ShadowToken
		durations: DurationToken
		easings: EasingToken
		animations: AnimationToken
		breakpoints: BreakpointToken
} & { [token: string]: never }

export type TokenCategory = "aspectRatios" | "zIndex" | "opacity" | "colors" | "fonts" | "fontSizes" | "fontWeights" | "lineHeights" | "letterSpacings" | "sizes" | "shadows" | "spacing" | "radii" | "borders" | "borderWidths" | "durations" | "easings" | "animations" | "blurs" | "gradients" | "breakpoints" | "assets"