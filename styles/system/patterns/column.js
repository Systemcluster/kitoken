import { getPatternStyles, patternFns } from '../helpers.js';
import { css } from '../css/index.js';

const columnConfig = {
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
defaultValues:{direction:'column',gap:'2',align:'flex-start',justify:'flex-start',wrap:'wrap',minWidth:'0%',maxWidth:'100%',width:'auto'}}

export const getColumnStyle = (styles = {}) => {
  const _styles = getPatternStyles(columnConfig, styles)
  return columnConfig.transform(_styles, patternFns)
}

export const column = (styles) => css(getColumnStyle(styles))
column.raw = getColumnStyle