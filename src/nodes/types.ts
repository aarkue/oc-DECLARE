import type { Node } from '@xyflow/react';

export type ActivityNode = Node<{ type: string, isObject?: "init"|"exit"}, 'activity'>;
export type AppNode =  ActivityNode;
