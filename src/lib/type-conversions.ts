import { CustomEdge, EdgeType } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { ReactFlowInstance } from "@xyflow/react";
import { OCDeclareArc } from "crates/shared/bindings/OCDeclareArc";
import { OCDeclareArcType } from "crates/shared/bindings/OCDeclareArcType";
import { get_edge_violation_percentage_perf } from "../../crates/backend-wasm/pkg/backend_wasm";

export function translateArcInfo(data: CustomEdge['data']): [OCDeclareArcType, [number | null, number | null]] {
    switch (data!.type) {
        case "ef":
            return ["EF", data?.cardinality ?? [1, null]]
        case "ef-rev":
            return ["EFREV", data?.cardinality ?? [1, null]]
        case "nef":
            return ["EF", [0, 0]]
        case "nef-rev":
            return ["EFREV", [0, 0]]
        case "ass":
            return ["ASS", data?.cardinality ?? [1, null]]
        case "df": return ["DF", data?.cardinality ?? [1, null]]
        case "df-rev": return ["DFREV", data?.cardinality ?? [1, null]]
        case "ndf": return ["DF", [0, 0]]
        case "ndf-rev": return ["DFREV", [0, 0]]
    };

}

export function translateArcTypeFromRsToTs(arcType: OCDeclareArcType): EdgeType {
    switch (arcType) {
        case "ASS":
            return "ass"
        case "EF":
            return "ef"
        case "EFREV":
            return "ef-rev"
        case "DF":
            return "df"
        case "DFREV":
            return "df-rev"
    }

}

export function flowEdgeToOCDECLARE(e: CustomEdge, flow: ReactFlowInstance<ActivityNode, CustomEdge>): OCDeclareArc {
const [arc_type, counts] = translateArcInfo(e.data!);
    return{
        from: flow.getNode(e.source)!.data.isObject === "init" ? { type: "ObjectInit", object_type: flow.getNode(e.source)!.data.type } : flow.getNode(e.source)!.data.isObject === "exit" ? { type: "ObjectEnd", object_type: flow.getNode(e.source)!.data.type } : { type: "Activity", activity: flow.getNode(e.source)!.data.type },
        to: flow.getNode(e.target)!.data.isObject === "init" ? { type: "ObjectInit", object_type: flow.getNode(e.target)!.data.type } : flow.getNode(e.target)!.data.isObject === "exit" ? { type: "ObjectEnd", object_type: flow.getNode(e.target)!.data.type } : { type: "Activity", activity: flow.getNode(e.target)!.data.type },
        arc_type,
        counts,
        label: e.data!.objectTypes
    };
}


export function getEdgeViolationPerc(arc: OCDeclareArc) : number {
    return  get_edge_violation_percentage_perf(JSON.stringify(arc))
}