/* eslint-disable */
import type { SystemStyleObject, ConditionalValue } from '../types/index';
import type { Properties } from '../types/csstype';
import type { SystemProperties } from '../types/style-props';
import type { DistributiveOmit } from '../types/system-types';
import type { Tokens } from '../tokens/index';

export interface FillProperties {
   
}


interface FillStyles extends FillProperties, DistributiveOmit<SystemStyleObject, keyof FillProperties > {}

interface FillPatternFn {
  (styles?: FillStyles): string
  raw: (styles?: FillStyles) => SystemStyleObject
}


export declare const fill: FillPatternFn;
