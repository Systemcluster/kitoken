import { getPatternStyles, patternFns } from '../helpers.js';
import { css } from '../css/index.js';

const rowConfig = {
transform(props) {
  const { direction, align, justify, wrap: wrap2, basis, grow, shrink, ...rest } = props;
  return {
    display: "flex",
    flexDirection: direction,
    alignItems: align,
    justifyContent: justify,
    flexWrap: wrap2,
    flexBasis: basis,
    flexGrow: grow,
    flexShrink: shrink,
    ...rest
  };
},
defaultValues:{direction:'row',gap:'2',align:'flex-start',justify:'flex-start',wrap:'wrap',minWidth:'0%',maxWidth:'100%',width:'auto'}}

export const getRowStyle = (styles = {}) => {
  const _styles = getPatternStyles(rowConfig, styles)
  return rowConfig.transform(_styles, patternFns)
}

export const row = (styles) => css(getRowStyle(styles))
row.raw = getRowStyle