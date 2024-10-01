const tokens = {
  "colors.transparent": {
    "value": "#ffffff00",
    "variable": "var(--colors-transparent)"
  },
  "colors.background.primary": {
    "value": "#fbfcfd",
    "variable": "var(--colors-background-primary)"
  },
  "colors.background.secondary": {
    "value": "#f2f3f4",
    "variable": "var(--colors-background-secondary)"
  },
  "colors.background.backdrop": {
    "value": "#eeeff0",
    "variable": "var(--colors-background-backdrop)"
  },
  "colors.background.overlay": {
    "value": "#f8f9fa",
    "variable": "var(--colors-background-overlay)"
  },
  "colors.background.muted": {
    "value": "#ecebea",
    "variable": "var(--colors-background-muted)"
  },
  "colors.background.inverse": {
    "value": "#0a0a0d",
    "variable": "var(--colors-background-inverse)"
  },
  "colors.text.primary": {
    "value": "#0a0a0d",
    "variable": "var(--colors-text-primary)"
  },
  "colors.text.secondary": {
    "value": "#17171a",
    "variable": "var(--colors-text-secondary)"
  },
  "colors.text.tertiary": {
    "value": "#3a3b3d",
    "variable": "var(--colors-text-tertiary)"
  },
  "colors.text.muted": {
    "value": "#525253",
    "variable": "var(--colors-text-muted)"
  },
  "colors.border.primary": {
    "value": "#dddddf",
    "variable": "var(--colors-border-primary)"
  },
  "colors.border.active": {
    "value": "#babbbc",
    "variable": "var(--colors-border-active)"
  },
  "colors.shadow.primary": {
    "value": "rgba(0, 0, 0, 0.1)",
    "variable": "var(--colors-shadow-primary)"
  },
  "colors.shadow.muted": {
    "value": "rgba(0, 0, 0, 0.05)",
    "variable": "var(--colors-shadow-muted)"
  },
  "colors.brand.primary.blue": {
    "value": "#5779b8",
    "variable": "var(--colors-brand-primary-blue)"
  },
  "colors.brand.primary.green": {
    "value": "#88cb68",
    "variable": "var(--colors-brand-primary-green)"
  },
  "colors.brand.primary.yellow": {
    "value": "#e9c815",
    "variable": "var(--colors-brand-primary-yellow)"
  },
  "colors.brand.primary.red": {
    "value": "#961d2a",
    "variable": "var(--colors-brand-primary-red)"
  },
  "colors.brand.primary.teal": {
    "value": "#aadbeb",
    "variable": "var(--colors-brand-primary-teal)"
  },
  "colors.brand.primary.purple": {
    "value": "#d49ed2",
    "variable": "var(--colors-brand-primary-purple)"
  },
  "colors.brand.primary.orange": {
    "value": "#dd5c3f",
    "variable": "var(--colors-brand-primary-orange)"
  },
  "colors.brand.primary.pink": {
    "value": "#ffa9c3",
    "variable": "var(--colors-brand-primary-pink)"
  },
  "colors.brand.primary.gold": {
    "value": "#f2e8b8",
    "variable": "var(--colors-brand-primary-gold)"
  },
  "colors.brand.secondary.blue": {
    "value": "#6b96e5",
    "variable": "var(--colors-brand-secondary-blue)"
  },
  "colors.brand.secondary.green": {
    "value": "#a9dd71",
    "variable": "var(--colors-brand-secondary-green)"
  },
  "colors.brand.secondary.yellow": {
    "value": "#ffd817",
    "variable": "var(--colors-brand-secondary-yellow)"
  },
  "colors.brand.secondary.red": {
    "value": "#ba2333",
    "variable": "var(--colors-brand-secondary-red)"
  },
  "colors.brand.secondary.teal": {
    "value": "#c7ffff",
    "variable": "var(--colors-brand-secondary-teal)"
  },
  "colors.brand.secondary.purple": {
    "value": "#e6abe3",
    "variable": "var(--colors-brand-secondary-purple)"
  },
  "colors.brand.secondary.orange": {
    "value": "#dd5c3f",
    "variable": "var(--colors-brand-secondary-orange)"
  },
  "colors.brand.secondary.pink": {
    "value": "#ffa9c3",
    "variable": "var(--colors-brand-secondary-pink)"
  },
  "colors.brand.secondary.gold": {
    "value": "#e5d8b2",
    "variable": "var(--colors-brand-secondary-gold)"
  },
  "fonts.main": {
    "value": "Satoshi Var, var(--font-system)",
    "variable": "var(--fonts-main)"
  },
  "fonts.head": {
    "value": "InterVariable, var(--font-system)",
    "variable": "var(--fonts-head)"
  },
  "fonts.text": {
    "value": "IAWriterQ Var, var(--font-system)",
    "variable": "var(--fonts-text)"
  },
  "fonts.mono": {
    "value": "Geist Mono Var, var(--font-mono)",
    "variable": "var(--fonts-mono)"
  },
  "fontWeights.main": {
    "value": 400,
    "variable": "var(--font-weights-main)"
  },
  "fontWeights.head": {
    "value": 700,
    "variable": "var(--font-weights-head)"
  },
  "fontWeights.thin": {
    "value": 200,
    "variable": "var(--font-weights-thin)"
  },
  "fontWeights.light": {
    "value": 300,
    "variable": "var(--font-weights-light)"
  },
  "fontWeights.regular": {
    "value": 400,
    "variable": "var(--font-weights-regular)"
  },
  "fontWeights.medium": {
    "value": 500,
    "variable": "var(--font-weights-medium)"
  },
  "fontWeights.bold": {
    "value": 600,
    "variable": "var(--font-weights-bold)"
  },
  "fontWeights.bolder": {
    "value": 700,
    "variable": "var(--font-weights-bolder)"
  },
  "fontWeights.black": {
    "value": 800,
    "variable": "var(--font-weights-black)"
  },
  "fontWeights.massive": {
    "value": 900,
    "variable": "var(--font-weights-massive)"
  },
  "spacing.xs": {
    "value": "2px",
    "variable": "var(--spacing-xs)"
  },
  "spacing.sm": {
    "value": "4px",
    "variable": "var(--spacing-sm)"
  },
  "spacing.md": {
    "value": "8px",
    "variable": "var(--spacing-md)"
  },
  "spacing.lg": {
    "value": "12px",
    "variable": "var(--spacing-lg)"
  },
  "spacing.xl": {
    "value": "16px",
    "variable": "var(--spacing-xl)"
  },
  "spacing.x2": {
    "value": "24px",
    "variable": "var(--spacing-x2)"
  },
  "spacing.x3": {
    "value": "32px",
    "variable": "var(--spacing-x3)"
  },
  "spacing.x4": {
    "value": "48px",
    "variable": "var(--spacing-x4)"
  },
  "spacing.x5": {
    "value": "64px",
    "variable": "var(--spacing-x5)"
  },
  "spacing.x6": {
    "value": "96px",
    "variable": "var(--spacing-x6)"
  },
  "spacing.x7": {
    "value": "128px",
    "variable": "var(--spacing-x7)"
  },
  "spacing.x8": {
    "value": "256px",
    "variable": "var(--spacing-x8)"
  },
  "sizes.xs": {
    "value": "2px",
    "variable": "var(--sizes-xs)"
  },
  "sizes.sm": {
    "value": "4px",
    "variable": "var(--sizes-sm)"
  },
  "sizes.md": {
    "value": "8px",
    "variable": "var(--sizes-md)"
  },
  "sizes.lg": {
    "value": "12px",
    "variable": "var(--sizes-lg)"
  },
  "sizes.xl": {
    "value": "16px",
    "variable": "var(--sizes-xl)"
  },
  "sizes.x2": {
    "value": "32px",
    "variable": "var(--sizes-x2)"
  },
  "sizes.x3": {
    "value": "64px",
    "variable": "var(--sizes-x3)"
  },
  "sizes.x4": {
    "value": "128px",
    "variable": "var(--sizes-x4)"
  },
  "sizes.x5": {
    "value": "256px",
    "variable": "var(--sizes-x5)"
  },
  "sizes.x6": {
    "value": "512px",
    "variable": "var(--sizes-x6)"
  },
  "sizes.x7": {
    "value": "1024px",
    "variable": "var(--sizes-x7)"
  },
  "sizes.x8": {
    "value": "1280px",
    "variable": "var(--sizes-x8)"
  },
  "sizes.breakpoint-sm": {
    "value": "768px",
    "variable": "var(--sizes-breakpoint-sm)"
  },
  "sizes.breakpoint-md": {
    "value": "1024px",
    "variable": "var(--sizes-breakpoint-md)"
  },
  "sizes.breakpoint-lg": {
    "value": "1312px",
    "variable": "var(--sizes-breakpoint-lg)"
  },
  "fontSizes.xs": {
    "value": "0.8125rem",
    "variable": "var(--font-sizes-xs)"
  },
  "fontSizes.sm": {
    "value": "0.875rem",
    "variable": "var(--font-sizes-sm)"
  },
  "fontSizes.md": {
    "value": "1rem",
    "variable": "var(--font-sizes-md)"
  },
  "fontSizes.lg": {
    "value": "1.25rem",
    "variable": "var(--font-sizes-lg)"
  },
  "fontSizes.xl": {
    "value": "1.5rem",
    "variable": "var(--font-sizes-xl)"
  },
  "fontSizes.x2": {
    "value": "2rem",
    "variable": "var(--font-sizes-x2)"
  },
  "fontSizes.x3": {
    "value": "3rem",
    "variable": "var(--font-sizes-x3)"
  },
  "fontSizes.x4": {
    "value": "4rem",
    "variable": "var(--font-sizes-x4)"
  },
  "fontSizes.x5": {
    "value": "5rem",
    "variable": "var(--font-sizes-x5)"
  },
  "fontSizes.x6": {
    "value": "6rem",
    "variable": "var(--font-sizes-x6)"
  },
  "lineHeights.xs": {
    "value": "0.8125",
    "variable": "var(--line-heights-xs)"
  },
  "lineHeights.sm": {
    "value": "0.875",
    "variable": "var(--line-heights-sm)"
  },
  "lineHeights.md": {
    "value": "1",
    "variable": "var(--line-heights-md)"
  },
  "lineHeights.lg": {
    "value": "1.25",
    "variable": "var(--line-heights-lg)"
  },
  "lineHeights.xl": {
    "value": "1.5",
    "variable": "var(--line-heights-xl)"
  },
  "lineHeights.x2": {
    "value": "2",
    "variable": "var(--line-heights-x2)"
  },
  "lineHeights.x3": {
    "value": "3",
    "variable": "var(--line-heights-x3)"
  },
  "lineHeights.x4": {
    "value": "4",
    "variable": "var(--line-heights-x4)"
  },
  "lineHeights.x5": {
    "value": "5",
    "variable": "var(--line-heights-x5)"
  },
  "lineHeights.x6": {
    "value": "8",
    "variable": "var(--line-heights-x6)"
  },
  "letterSpacings.xs": {
    "value": "-0.05em",
    "variable": "var(--letter-spacings-xs)"
  },
  "letterSpacings.sm": {
    "value": "-0.025em",
    "variable": "var(--letter-spacings-sm)"
  },
  "letterSpacings.md": {
    "value": "0em",
    "variable": "var(--letter-spacings-md)"
  },
  "letterSpacings.lg": {
    "value": "0.025em",
    "variable": "var(--letter-spacings-lg)"
  },
  "letterSpacings.xl": {
    "value": "0.05em",
    "variable": "var(--letter-spacings-xl)"
  },
  "letterSpacings.x2": {
    "value": "0.075em",
    "variable": "var(--letter-spacings-x2)"
  },
  "letterSpacings.x3": {
    "value": "0.1em",
    "variable": "var(--letter-spacings-x3)"
  },
  "letterSpacings.x4": {
    "value": "0.125em",
    "variable": "var(--letter-spacings-x4)"
  },
  "letterSpacings.x5": {
    "value": "0.15em",
    "variable": "var(--letter-spacings-x5)"
  },
  "letterSpacings.x6": {
    "value": "0.175em",
    "variable": "var(--letter-spacings-x6)"
  },
  "radii.full": {
    "value": "100%",
    "variable": "var(--radii-full)"
  },
  "radii.xs": {
    "value": "2px",
    "variable": "var(--radii-xs)"
  },
  "radii.sm": {
    "value": "4px",
    "variable": "var(--radii-sm)"
  },
  "radii.md": {
    "value": "8px",
    "variable": "var(--radii-md)"
  },
  "radii.lg": {
    "value": "12px",
    "variable": "var(--radii-lg)"
  },
  "radii.xl": {
    "value": "16px",
    "variable": "var(--radii-xl)"
  },
  "borderWidths.none": {
    "value": "0px",
    "variable": "var(--border-widths-none)"
  },
  "borderWidths.xs": {
    "value": "1px",
    "variable": "var(--border-widths-xs)"
  },
  "borderWidths.sm": {
    "value": "2px",
    "variable": "var(--border-widths-sm)"
  },
  "borderWidths.md": {
    "value": "4px",
    "variable": "var(--border-widths-md)"
  },
  "borderWidths.lg": {
    "value": "8px",
    "variable": "var(--border-widths-lg)"
  },
  "borderWidths.xl": {
    "value": "16px",
    "variable": "var(--border-widths-xl)"
  },
  "shadows.none": {
    "value": "none",
    "variable": "var(--shadows-none)"
  },
  "shadows.xs": {
    "value": "0px 1px 2px var(--colors-shadow-muted)",
    "variable": "var(--shadows-xs)"
  },
  "shadows.sm": {
    "value": "0px 1px 3px var(--colors-shadow-primary)",
    "variable": "var(--shadows-sm)"
  },
  "shadows.md": {
    "value": "0px 4px 6px var(--colors-shadow-primary)",
    "variable": "var(--shadows-md)"
  },
  "shadows.lg": {
    "value": "0px 10px 15px var(--colors-shadow-primary)",
    "variable": "var(--shadows-lg)"
  },
  "shadows.xl": {
    "value": "0px 20px 25px var(--colors-shadow-primary)",
    "variable": "var(--shadows-xl)"
  },
  "durations.none": {
    "value": "0ms",
    "variable": "var(--durations-none)"
  },
  "durations.fast": {
    "value": "0.08s",
    "variable": "var(--durations-fast)"
  },
  "durations.normal": {
    "value": "0.14s",
    "variable": "var(--durations-normal)"
  },
  "durations.medium": {
    "value": "0.22s",
    "variable": "var(--durations-medium)"
  },
  "durations.slow": {
    "value": "0.36s",
    "variable": "var(--durations-slow)"
  },
  "durations.slower": {
    "value": "0.58s",
    "variable": "var(--durations-slower)"
  },
  "durations.slowest": {
    "value": "0.94s",
    "variable": "var(--durations-slowest)"
  },
  "easings.none": {
    "value": "none",
    "variable": "var(--easings-none)"
  },
  "easings.linear": {
    "value": "cubic-bezier(0.25, 0.25, 0.75, 0.75)",
    "variable": "var(--easings-linear)"
  },
  "easings.ease": {
    "value": "cubic-bezier(0.25, 0.1, 0.25, 1)",
    "variable": "var(--easings-ease)"
  },
  "easings.easeSmooth": {
    "value": "cubic-bezier(0.48, 0.205, 0.295, 0.92)",
    "variable": "var(--easings-ease-smooth)"
  },
  "easings.easeIn": {
    "value": "cubic-bezier(0.42, 0, 1, 1)",
    "variable": "var(--easings-ease-in)"
  },
  "easings.easeInQuad": {
    "value": "cubic-bezier(0.55, 0.085, 0.68, 0.53)",
    "variable": "var(--easings-ease-in-quad)"
  },
  "easings.easeInCubic": {
    "value": "cubic-bezier(0.55, 0.055, 0.675, 0.19)",
    "variable": "var(--easings-ease-in-cubic)"
  },
  "easings.easeInQuart": {
    "value": "cubic-bezier(0.895, 0.03, 0.685, 0.22)",
    "variable": "var(--easings-ease-in-quart)"
  },
  "easings.easeInQuint": {
    "value": "cubic-bezier(0.755, 0.05, 0.855, 0.06)",
    "variable": "var(--easings-ease-in-quint)"
  },
  "easings.easeInSine": {
    "value": "cubic-bezier(0.47, 0, 0.745, 0.715)",
    "variable": "var(--easings-ease-in-sine)"
  },
  "easings.easeInExpo": {
    "value": "cubic-bezier(0.95, 0.05, 0.795, 0.035)",
    "variable": "var(--easings-ease-in-expo)"
  },
  "easings.easeInCirc": {
    "value": "cubic-bezier(0.6, 0.04, 0.98, 0.335)",
    "variable": "var(--easings-ease-in-circ)"
  },
  "easings.easeInBack": {
    "value": "cubic-bezier(0.6, -0.28, 0.735, 0.045)",
    "variable": "var(--easings-ease-in-back)"
  },
  "easings.easeOut": {
    "value": "cubic-bezier(0, 0, 0.58, 1)",
    "variable": "var(--easings-ease-out)"
  },
  "easings.easeOutQuad": {
    "value": "cubic-bezier(0.25, 0.46, 0.45, 0.94)",
    "variable": "var(--easings-ease-out-quad)"
  },
  "easings.easeOutCubic": {
    "value": "cubic-bezier(0.215, 0.61, 0.355, 1)",
    "variable": "var(--easings-ease-out-cubic)"
  },
  "easings.easeOutQuart": {
    "value": "cubic-bezier(0.165, 0.84, 0.44, 1)",
    "variable": "var(--easings-ease-out-quart)"
  },
  "easings.easeOutQuint": {
    "value": "cubic-bezier(0.23, 1, 0.32, 1)",
    "variable": "var(--easings-ease-out-quint)"
  },
  "easings.easeOutSine": {
    "value": "cubic-bezier(0.39, 0.575, 0.565, 1)",
    "variable": "var(--easings-ease-out-sine)"
  },
  "easings.easeOutExpo": {
    "value": "cubic-bezier(0.19, 1, 0.22, 1)",
    "variable": "var(--easings-ease-out-expo)"
  },
  "easings.easeOutCirc": {
    "value": "cubic-bezier(0.075, 0.82, 0.165, 1)",
    "variable": "var(--easings-ease-out-circ)"
  },
  "easings.easeOutBack": {
    "value": "cubic-bezier(0.175, 0.885, 0.32, 1.275)",
    "variable": "var(--easings-ease-out-back)"
  },
  "easings.easeInOut": {
    "value": "cubic-bezier(0.42, 0, 0.58, 1)",
    "variable": "var(--easings-ease-in-out)"
  },
  "easings.easeInOutQuad": {
    "value": "cubic-bezier(0.455, 0.03, 0.515, 0.955)",
    "variable": "var(--easings-ease-in-out-quad)"
  },
  "easings.easeInOutCubic": {
    "value": "cubic-bezier(0.645, 0.045, 0.355, 1)",
    "variable": "var(--easings-ease-in-out-cubic)"
  },
  "easings.easeInOutQuart": {
    "value": "cubic-bezier(0.77, 0, 0.175, 1)",
    "variable": "var(--easings-ease-in-out-quart)"
  },
  "easings.easeInOutQuint": {
    "value": "cubic-bezier(0.86, 0, 0.07, 1)",
    "variable": "var(--easings-ease-in-out-quint)"
  },
  "easings.easeInOutSine": {
    "value": "cubic-bezier(0.445, 0.05, 0.55, 0.95)",
    "variable": "var(--easings-ease-in-out-sine)"
  },
  "easings.easeInOutExpo": {
    "value": "cubic-bezier(1, 0, 0, 1)",
    "variable": "var(--easings-ease-in-out-expo)"
  },
  "easings.easeInOutCirc": {
    "value": "cubic-bezier(0.785, 0.135, 0.15, 0.86)",
    "variable": "var(--easings-ease-in-out-circ)"
  },
  "easings.easeInOutBack": {
    "value": "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
    "variable": "var(--easings-ease-in-out-back)"
  },
  "animations.spin": {
    "value": "spin var(--durations-slower) linear infinite",
    "variable": "var(--animations-spin)"
  },
  "animations.fadeIn": {
    "value": "fadeIn var(--durations-slower) var(--easings-ease-in)",
    "variable": "var(--animations-fade-in)"
  },
  "animations.scrollX": {
    "value": "scrollX var(--durations-slower) linear infinite",
    "variable": "var(--animations-scroll-x)"
  },
  "animations.scrollY": {
    "value": "scrollY var(--durations-slower) linear infinite",
    "variable": "var(--animations-scroll-y)"
  },
  "animations.flickerX": {
    "value": "flickerX var(--durations-fast) linear infinite",
    "variable": "var(--animations-flicker-x)"
  },
  "animations.flickerY": {
    "value": "flickerY var(--durations-fast) linear infinite",
    "variable": "var(--animations-flicker-y)"
  },
  "animations.moveDown": {
    "value": "moveDown var(--durations-normal) var(--easings-ease-in)",
    "variable": "var(--animations-move-down)"
  },
  "animations.moveUp": {
    "value": "moveUp var(--durations-normal) var(--easings-ease-in)",
    "variable": "var(--animations-move-up)"
  },
  "animations.moveLeft": {
    "value": "moveLeft var(--durations-normal) var(--easings-ease-in)",
    "variable": "var(--animations-move-left)"
  },
  "animations.moveRight": {
    "value": "moveRight var(--durations-normal) var(--easings-ease-in)",
    "variable": "var(--animations-move-right)"
  },
  "animations.scaleIn": {
    "value": "scaleIn var(--durations-slowest) var(--easings-ease-out)",
    "variable": "var(--animations-scale-in)"
  },
  "animations.scaleOut": {
    "value": "scaleOut var(--durations-slowest) var(--easings-ease-in)",
    "variable": "var(--animations-scale-out)"
  },
  "breakpoints.sm": {
    "value": "768px",
    "variable": "var(--breakpoints-sm)"
  },
  "breakpoints.md": {
    "value": "1024px",
    "variable": "var(--breakpoints-md)"
  },
  "breakpoints.lg": {
    "value": "1312px",
    "variable": "var(--breakpoints-lg)"
  },
  "spacing.-xs": {
    "value": "calc(var(--spacing-xs) * -1)",
    "variable": "var(--spacing-xs)"
  },
  "spacing.-sm": {
    "value": "calc(var(--spacing-sm) * -1)",
    "variable": "var(--spacing-sm)"
  },
  "spacing.-md": {
    "value": "calc(var(--spacing-md) * -1)",
    "variable": "var(--spacing-md)"
  },
  "spacing.-lg": {
    "value": "calc(var(--spacing-lg) * -1)",
    "variable": "var(--spacing-lg)"
  },
  "spacing.-xl": {
    "value": "calc(var(--spacing-xl) * -1)",
    "variable": "var(--spacing-xl)"
  },
  "spacing.-x2": {
    "value": "calc(var(--spacing-x2) * -1)",
    "variable": "var(--spacing-x2)"
  },
  "spacing.-x3": {
    "value": "calc(var(--spacing-x3) * -1)",
    "variable": "var(--spacing-x3)"
  },
  "spacing.-x4": {
    "value": "calc(var(--spacing-x4) * -1)",
    "variable": "var(--spacing-x4)"
  },
  "spacing.-x5": {
    "value": "calc(var(--spacing-x5) * -1)",
    "variable": "var(--spacing-x5)"
  },
  "spacing.-x6": {
    "value": "calc(var(--spacing-x6) * -1)",
    "variable": "var(--spacing-x6)"
  },
  "spacing.-x7": {
    "value": "calc(var(--spacing-x7) * -1)",
    "variable": "var(--spacing-x7)"
  },
  "spacing.-x8": {
    "value": "calc(var(--spacing-x8) * -1)",
    "variable": "var(--spacing-x8)"
  },
  "colors.colorPalette": {
    "value": "var(--colors-color-palette)",
    "variable": "var(--colors-color-palette)"
  },
  "colors.colorPalette.primary": {
    "value": "var(--colors-color-palette-primary)",
    "variable": "var(--colors-color-palette-primary)"
  },
  "colors.colorPalette.secondary": {
    "value": "var(--colors-color-palette-secondary)",
    "variable": "var(--colors-color-palette-secondary)"
  },
  "colors.colorPalette.backdrop": {
    "value": "var(--colors-color-palette-backdrop)",
    "variable": "var(--colors-color-palette-backdrop)"
  },
  "colors.colorPalette.overlay": {
    "value": "var(--colors-color-palette-overlay)",
    "variable": "var(--colors-color-palette-overlay)"
  },
  "colors.colorPalette.muted": {
    "value": "var(--colors-color-palette-muted)",
    "variable": "var(--colors-color-palette-muted)"
  },
  "colors.colorPalette.inverse": {
    "value": "var(--colors-color-palette-inverse)",
    "variable": "var(--colors-color-palette-inverse)"
  },
  "colors.colorPalette.tertiary": {
    "value": "var(--colors-color-palette-tertiary)",
    "variable": "var(--colors-color-palette-tertiary)"
  },
  "colors.colorPalette.active": {
    "value": "var(--colors-color-palette-active)",
    "variable": "var(--colors-color-palette-active)"
  },
  "colors.colorPalette.primary.blue": {
    "value": "var(--colors-color-palette-primary-blue)",
    "variable": "var(--colors-color-palette-primary-blue)"
  },
  "colors.colorPalette.blue": {
    "value": "var(--colors-color-palette-blue)",
    "variable": "var(--colors-color-palette-blue)"
  },
  "colors.colorPalette.primary.green": {
    "value": "var(--colors-color-palette-primary-green)",
    "variable": "var(--colors-color-palette-primary-green)"
  },
  "colors.colorPalette.green": {
    "value": "var(--colors-color-palette-green)",
    "variable": "var(--colors-color-palette-green)"
  },
  "colors.colorPalette.primary.yellow": {
    "value": "var(--colors-color-palette-primary-yellow)",
    "variable": "var(--colors-color-palette-primary-yellow)"
  },
  "colors.colorPalette.yellow": {
    "value": "var(--colors-color-palette-yellow)",
    "variable": "var(--colors-color-palette-yellow)"
  },
  "colors.colorPalette.primary.red": {
    "value": "var(--colors-color-palette-primary-red)",
    "variable": "var(--colors-color-palette-primary-red)"
  },
  "colors.colorPalette.red": {
    "value": "var(--colors-color-palette-red)",
    "variable": "var(--colors-color-palette-red)"
  },
  "colors.colorPalette.primary.teal": {
    "value": "var(--colors-color-palette-primary-teal)",
    "variable": "var(--colors-color-palette-primary-teal)"
  },
  "colors.colorPalette.teal": {
    "value": "var(--colors-color-palette-teal)",
    "variable": "var(--colors-color-palette-teal)"
  },
  "colors.colorPalette.primary.purple": {
    "value": "var(--colors-color-palette-primary-purple)",
    "variable": "var(--colors-color-palette-primary-purple)"
  },
  "colors.colorPalette.purple": {
    "value": "var(--colors-color-palette-purple)",
    "variable": "var(--colors-color-palette-purple)"
  },
  "colors.colorPalette.primary.orange": {
    "value": "var(--colors-color-palette-primary-orange)",
    "variable": "var(--colors-color-palette-primary-orange)"
  },
  "colors.colorPalette.orange": {
    "value": "var(--colors-color-palette-orange)",
    "variable": "var(--colors-color-palette-orange)"
  },
  "colors.colorPalette.primary.pink": {
    "value": "var(--colors-color-palette-primary-pink)",
    "variable": "var(--colors-color-palette-primary-pink)"
  },
  "colors.colorPalette.pink": {
    "value": "var(--colors-color-palette-pink)",
    "variable": "var(--colors-color-palette-pink)"
  },
  "colors.colorPalette.primary.gold": {
    "value": "var(--colors-color-palette-primary-gold)",
    "variable": "var(--colors-color-palette-primary-gold)"
  },
  "colors.colorPalette.gold": {
    "value": "var(--colors-color-palette-gold)",
    "variable": "var(--colors-color-palette-gold)"
  },
  "colors.colorPalette.secondary.blue": {
    "value": "var(--colors-color-palette-secondary-blue)",
    "variable": "var(--colors-color-palette-secondary-blue)"
  },
  "colors.colorPalette.secondary.green": {
    "value": "var(--colors-color-palette-secondary-green)",
    "variable": "var(--colors-color-palette-secondary-green)"
  },
  "colors.colorPalette.secondary.yellow": {
    "value": "var(--colors-color-palette-secondary-yellow)",
    "variable": "var(--colors-color-palette-secondary-yellow)"
  },
  "colors.colorPalette.secondary.red": {
    "value": "var(--colors-color-palette-secondary-red)",
    "variable": "var(--colors-color-palette-secondary-red)"
  },
  "colors.colorPalette.secondary.teal": {
    "value": "var(--colors-color-palette-secondary-teal)",
    "variable": "var(--colors-color-palette-secondary-teal)"
  },
  "colors.colorPalette.secondary.purple": {
    "value": "var(--colors-color-palette-secondary-purple)",
    "variable": "var(--colors-color-palette-secondary-purple)"
  },
  "colors.colorPalette.secondary.orange": {
    "value": "var(--colors-color-palette-secondary-orange)",
    "variable": "var(--colors-color-palette-secondary-orange)"
  },
  "colors.colorPalette.secondary.pink": {
    "value": "var(--colors-color-palette-secondary-pink)",
    "variable": "var(--colors-color-palette-secondary-pink)"
  },
  "colors.colorPalette.secondary.gold": {
    "value": "var(--colors-color-palette-secondary-gold)",
    "variable": "var(--colors-color-palette-secondary-gold)"
  }
}

export function token(path, fallback) {
  return tokens[path]?.value || fallback
}

function tokenVar(path, fallback) {
  return tokens[path]?.variable || fallback
}

token.var = tokenVar