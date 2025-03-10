import { CustomEdge, EdgeType, getMarkersForEdge } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { ReactFlowInstance } from "@xyflow/react";
import { OCDeclareArc } from "crates/shared/bindings/OCDeclareArc";
import { OCDeclareArcType } from "crates/shared/bindings/OCDeclareArcType";
import { get_edge_violation_percentage_perf } from "../../crates/backend-wasm/pkg/backend_wasm";
import { OCDeclareNode } from "crates/shared/bindings/OCDeclareNode";

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
    return {
        from: flow.getNode(e.source)!.data.isObject === "init" ? { type: "ObjectInit", object_type: flow.getNode(e.source)!.data.type } : flow.getNode(e.source)!.data.isObject === "exit" ? { type: "ObjectEnd", object_type: flow.getNode(e.source)!.data.type } : { type: "Activity", activity: flow.getNode(e.source)!.data.type },
        to: flow.getNode(e.target)!.data.isObject === "init" ? { type: "ObjectInit", object_type: flow.getNode(e.target)!.data.type } : flow.getNode(e.target)!.data.isObject === "exit" ? { type: "ObjectEnd", object_type: flow.getNode(e.target)!.data.type } : { type: "Activity", activity: flow.getNode(e.target)!.data.type },
        arc_type,
        counts,
        label: e.data!.objectTypes
    };
}


export function getEdgeViolationPerc(arc: OCDeclareArc): number {
    return get_edge_violation_percentage_perf(JSON.stringify(arc))
}

import { v4 as uuidv4 } from 'uuid';
import { applyLayoutToNodes } from "./automatic-layout";
export async function addArcsToFlow(discoverdArcs: OCDeclareArc[], flow: ReactFlowInstance<ActivityNode, CustomEdge>) {
    const nodeNameToIDs: Record<string, string> = {};
    const edges: CustomEdge[] = [];
    const nodes: ActivityNode[] = [];
    for (const arc of discoverdArcs) {
        const edgeType = translateArcTypeFromRsToTs(arc.arc_type);
        // const NON_RESOURCE_TYPES = ["orders", "items", "packages"];
        // const isNotOnlyResource = arc.label.all.map(oi => getLastOT(oi)).find(ot => NON_RESOURCE_TYPES.includes(ot)) || arc.label.each.map(oi => getLastOT(oi)).find(ot => NON_RESOURCE_TYPES.includes(ot)) || arc.label.any.map(oi => getLastOT(oi)).find(ot => NON_RESOURCE_TYPES.includes(ot));
        const sourceID = lookupIDOrCreateNode(arc.from, nodeNameToIDs, nodes);
        const targetID = lookupIDOrCreateNode(arc.to, nodeNameToIDs, nodes);
        const edgeID = uuidv4();
        edges.push({ id: edgeID, source: sourceID, target: targetID, data: { type: edgeType, objectTypes: arc.label, cardinality: arc.counts }, ...getMarkersForEdge(edgeType, edgeID) })
    }
    applyLayoutToNodes(nodes, edges).then(() => {
        flow.addNodes(nodes);
        flow.addEdges(edges);
    })
}

// function getLastOT(otass: ObjectTypeAssociation) {
//     if (otass.type === "Simple") {
//         return otass.object_type
//     } else {
//         return otass.second;
//     }
// }

function lookupIDOrCreateNode(node: OCDeclareNode, nodeIDMap: Record<string, string>, nodes: ActivityNode[]): string {
    let nodeName = node.type === "Activity" ? node.activity : (node.type === "ObjectInit" ? "<init> " + node.object_type : "<exit> " + node.object_type);
    let isObject: ActivityNode['data']['isObject'] = undefined;
    if (node.type === "Activity") {
        if (node.activity.includes("<init> ")) {
            isObject = "init";
        } else if (node.activity.includes("<exit> ")) {
            isObject = "exit";
        }
    } else if (node.type === "ObjectInit") {
        isObject = "init"
    } else {
        isObject = "exit"
    }
    if (nodeIDMap[nodeName] == undefined) {
        const id = uuidv4();
        nodes.push({ id: id, type: "activity", position: { x: 0, y: 0 }, data: { isObject, type: node.type === "Activity" ? node.activity.replace("<init> ", "").replace("<exit> ", "") : node.object_type } })
        nodeIDMap[nodeName] = id;
        return id;
    } else {
        return nodeIDMap[nodeName];
    }
}

