/* eslint-disable */
import type { FunctionComponent } from 'react'
import type { FillProperties } from '../patterns/fill';
import type { HTMLStyledProps } from '../types/jsx';
import type { DistributiveOmit } from '../types/system-types';

export interface FillProps extends FillProperties, DistributiveOmit<HTMLStyledProps<'div'>, keyof FillProperties > {}


export declare const Fill: FunctionComponent<FillProps>