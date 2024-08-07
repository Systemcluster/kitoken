/* eslint-disable */
import type { FunctionComponent } from 'react'
import type { ColumnProperties } from '../patterns/column';
import type { HTMLStyledProps } from '../types/jsx';
import type { DistributiveOmit } from '../types/system-types';

export interface ColumnProps extends ColumnProperties, DistributiveOmit<HTMLStyledProps<'div'>, keyof ColumnProperties > {}


export declare const Column: FunctionComponent<ColumnProps>