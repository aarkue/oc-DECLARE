import { type EdgeTypes } from '@xyflow/react';
import CustomEdge from './CustomEdge';
import { CustomEdge as CustomEdgeType } from './types';

export const initialEdges: CustomEdgeType[] = [
];

export const edgeTypes = {
  // Add your custom edge types here!
  "default": CustomEdge
} satisfies EdgeTypes;
