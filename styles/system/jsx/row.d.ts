/* eslint-disable */
import type { FunctionComponent } from 'react'
import type { RowProperties } from '../patterns/row';
import type { HTMLStyledProps } from '../types/jsx';
import type { DistributiveOmit } from '../types/system-types';

export interface RowProps extends RowProperties, DistributiveOmit<HTMLStyledProps<'div'>, keyof RowProperties > {}


export declare const Row: FunctionComponent<RowProps>