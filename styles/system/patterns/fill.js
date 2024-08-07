import { getPatternStyles, patternFns } from '../helpers.js';
import { css } from '../css/index.js';

const fillConfig = {
transform(props) {
  return props;
},
defaultValues:{width:'100%',height:'100%',top:'0',left:'0',position:'absolute'}}

export const getFillStyle = (styles = {}) => {
  const _styles = getPatternStyles(fillConfig, styles)
  return fillConfig.transform(_styles, patternFns)
}

export const fill = (styles) => css(getFillStyle(styles))
fill.raw = getFillStyle