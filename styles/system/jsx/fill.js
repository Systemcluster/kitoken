import { createElement, forwardRef } from 'react'
import { mergeCss } from '../css/css.js';
import { splitProps } from '../helpers.js';
import { getFillStyle } from '../patterns/fill.js';
import { styled } from './factory.js';

export const Fill = /* @__PURE__ */ forwardRef(function Fill(props, ref) {
  const [patternProps, restProps] = splitProps(props, [])

const styleProps = getFillStyle(patternProps)
const cssProps = { css: mergeCss(styleProps, props.css) }
const mergedProps = { ref, ...restProps, ...cssProps }

return createElement(styled.div, mergedProps)
  })