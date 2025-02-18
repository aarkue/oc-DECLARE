import type { Edge } from '@xyflow/react';
import { OCDeclareArcLabel } from 'crates/shared/bindings/OCDeclareArcLabel';

export const ALL_EDGE_TYPES = [
    // "ass",
    "ef",
    "ef-rev",
    "nef",
    "nef-rev",
    "df",
    "df-rev",
    "ndf",
    "ndf-rev",
    "ass"] as const;
export type EdgeType = typeof ALL_EDGE_TYPES[number];
export type CustomEdge = Edge<{ type: EdgeType, objectTypes: OCDeclareArcLabel, cardinality?: [number | null, number | null], violationInfo?: { violationPercentage: number } }>;
export type AppNode = CustomEdge;


const STROKE_WIDTH = 2.5;

export function getMarkersForEdge(edgeType: EdgeType, id?: string): { markerStart: string, markerEnd: string | undefined, style: React.CSSProperties } {
    if (edgeType === "ef") {
        return {
            markerStart: `start-${id}`,
            markerEnd: "single-arrow-marker",
            style: { stroke: "var(--arrow-primary)", strokeWidth: STROKE_WIDTH }
        }
    }
    if (edgeType === "ass") {
        return {
            markerStart: `start-${id}`,
            markerEnd: undefined,
            style: {
                stroke: "var(--arrow-primary)", strokeWidth: STROKE_WIDTH,
                //  strokeDasharray: "5 5" 
            }
        }
    }
    if (edgeType === "nef") {
        return {
            markerStart: `start-${id}`,
            markerEnd: "single-not-arrow-marker",
            style: { stroke: "var(--arrow-primary)", strokeWidth: STROKE_WIDTH }

        }
    }
    if (edgeType === "nef-rev") {
        return {

            markerStart: `start-${id}`,
            // markerStart: "single-not-arrow-marker-rev",
            markerEnd: undefined,
            style: { stroke: "var(--arrow-primary)", strokeWidth: STROKE_WIDTH }

        }
    }
    if (edgeType === "ef-rev") {
        return {
            markerStart: `start-${id}`,
            // markerStart: "single-arrow-marker-rev",
            markerEnd: undefined,
            style: { stroke: "var(--arrow-primary)", strokeWidth: STROKE_WIDTH }

        }
    }
    if (edgeType === "df") {
        return {
            markerStart: `start-${id}`,
            // markerStart: "single-arrow-marker-rev",
            markerEnd: "single-arrow-direct-marker",
            style: { stroke: "var(--arrow-primary)", strokeWidth: STROKE_WIDTH }
        }
    }
    if (edgeType === "df-rev" || edgeType === "ndf-rev") {
        return {
            markerStart: `start-${id}`,
            // markerStart: "single-arrow-marker-rev",
            markerEnd: undefined,
            style: { stroke: "var(--arrow-primary)", strokeWidth: STROKE_WIDTH }

        }
    }
    if (edgeType === "ndf") {
        return {
            markerStart: `start-${id}`,
            // markerStart: "single-arrow-marker-rev",
            markerEnd: "single-not-arrow-direct-marker",
            style: { stroke: "var(--arrow-primary)", strokeWidth: STROKE_WIDTH }

        }
    }
    return {
        markerStart: `start-${id}`,
        markerEnd: undefined,
        style: { stroke: "purple", strokeWidth: 2, strokeDasharray: "5 5" }
    }
}