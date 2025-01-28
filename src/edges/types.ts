import type { Edge } from '@xyflow/react';

export const ALL_EDGE_TYPES = [
    // "ass",
    "ef", "ef-rev", "nef", "nef-rev","ass"] as const;
export type EdgeType = typeof ALL_EDGE_TYPES[number];
export type CustomEdge = Edge<{ type: EdgeType, objectTypes?: (string | [string, string])[], cardinality?: [number | null, number | null] }>;
export type AppNode = CustomEdge;




export function getMarkersForEdge(edgeType: EdgeType): { markerStart: string, markerEnd: string | undefined, style: React.CSSProperties } {
    if (edgeType === "ef") {
        return {
            markerStart: "dot-marker",
            markerEnd: "single-arrow-marker",
            style: { stroke: "var(--arrow-primary)", strokeWidth: 2 }
        }
    }
    if (edgeType === "ass") {
        return {
            markerStart: "dot-marker",
            markerEnd: undefined,
            style: { stroke: "var(--arrow-primary)", strokeWidth: 2,
                //  strokeDasharray: "5 5" 
                }
        }
    }
    if (edgeType === "nef") {
        return {
            markerStart: "dot-marker",
            markerEnd: "single-not-arrow-marker",
            style: { stroke: "var(--arrow-primary)", strokeWidth: 2 }

        }
    }
    if (edgeType === "nef-rev") {
        return {
            markerStart: "single-not-arrow-marker-rev",
            markerEnd: undefined,
            style: { stroke: "var(--arrow-primary)", strokeWidth: 2 }

        }
    }
    if (edgeType === "ef-rev") {
        return {
            markerStart: "single-arrow-marker-rev",
            markerEnd: undefined,
            style: { stroke: "var(--arrow-primary)", strokeWidth: 2 }

        }
    }
    return {
        markerStart: "dot-marker",
        markerEnd: undefined,
        style: { stroke: "purple", strokeWidth: 2, strokeDasharray: "5 5" }
    }
}