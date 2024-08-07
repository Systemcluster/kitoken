/* eslint-disable */
import type { SystemStyleObject, ConditionalValue } from '../types/index';
import type { Properties } from '../types/csstype';
import type { SystemProperties } from '../types/style-props';
import type { DistributiveOmit } from '../types/system-types';
import type { Tokens } from '../tokens/index';

export interface RowProperties {
   align?: SystemProperties["alignItems"]
	justify?: SystemProperties["justifyContent"]
	direction?: SystemProperties["flexDirection"]
	wrap?: SystemProperties["flexWrap"]
	basis?: SystemProperties["flexBasis"]
	grow?: SystemProperties["flexGrow"]
	shrink?: SystemProperties["flexShrink"]
}


interface RowStyles extends RowProperties, DistributiveOmit<SystemStyleObject, keyof RowProperties > {}

interface RowPatternFn {
  (styles?: RowStyles): string
  raw: (styles?: RowStyles) => SystemStyleObject
}


export declare const row: RowPatternFn;
