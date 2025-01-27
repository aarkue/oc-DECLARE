import type { Node } from '@xyflow/react';

export type ActivityNode = Node<{ type: string, isObject?: boolean}, 'activity'>;
export type AppNode =  ActivityNode;
