import type { NodeTypes } from '@xyflow/react';

import { ActivityNode } from './ActivityNode';
import { AppNode } from './types';

export const initialNodes: AppNode[] = [

];

export const nodeTypes = {
  'activity': ActivityNode,
  // Add any of your custom nodes here!
} satisfies NodeTypes;
