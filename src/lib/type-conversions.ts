import { CustomEdge, EdgeType, getMarkersForEdge } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { ReactFlowInstance } from "@xyflow/react";
import { OCDeclareArc } from "crates/shared/bindings/OCDeclareArc";
import { OCDeclareArcType } from "crates/shared/bindings/OCDeclareArcType";
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
        from: flow.getNode(e.source)!.data.isObject === "init" ? "<init> " + flow.getNode(e.source)!.data.type : flow.getNode(e.source)!.data.isObject === "exit" ? "<exit> " + flow.getNode(e.source)!.data.type : flow.getNode(e.source)!.data.type,
        to: flow.getNode(e.target)!.data.isObject === "init" ? "<init> " + flow.getNode(e.target)!.data.type : flow.getNode(e.target)!.data.isObject === "exit" ? "<exit> " + flow.getNode(e.target)!.data.type : flow.getNode(e.target)!.data.type,
        arc_type,
        counts,
        label: e.data!.objectTypes
    };
}



import { v4 as uuidv4 } from 'uuid';
import { applyLayoutToNodes } from "./automatic-layout";
// import { ObjectTypeAssociation } from "crates/shared/bindings/ObjectTypeAssociation";
export async function addArcsToFlow(discoverdArcs: OCDeclareArc[], flow: ReactFlowInstance<ActivityNode, CustomEdge>) {
    const nodeNameToIDs: Record<string, string> = {};
    const edges: CustomEdge[] = [];
    const nodes: ActivityNode[] = [];
    for (const arc of discoverdArcs) {
        const edgeType = translateArcTypeFromRsToTs(arc.arc_type);
        // const NON_RESOURCE_TYPES = ["orders", "items", "packages","Offer","Application"];
        // const isNotOnlyResource = arc.label.all.map(oi => getLastOT(oi)).find(ot => NON_RESOURCE_TYPES.includes(ot)) || arc.label.each.map(oi => getLastOT(oi)).find(ot => NON_RESOURCE_TYPES.includes(ot)) || arc.label.any.map(oi => getLastOT(oi)).find(ot => NON_RESOURCE_TYPES.includes(ot));
        // if(isNotOnlyResource){
        // const from = typeof arc.from === "object" ? arc.from['activity'] : arc.from;
        // const to = typeof arc.to === "object" ? arc.to['activity'] : arc.to;
        // const flag = (from === "place order" && to === "confirm order")
        //     || (from === "confirm order" && to === "pick item")
        //     || (from === "pay order" && to === "pick item")
        //     || (from === "confirm order" && to === "pay order")
        //     || (from === "pick item" && to === "send package")
        //     || (from === "send package" && to === "package delivered")
        //     || (from === "payment reminder" && to === "package delivered")
        //     || (from === "A_Cancelled" && to === "O_Cancelled")
        // if (flag) {
            const sourceID = lookupIDOrCreateNode(arc.from, nodeNameToIDs, nodes);
            const targetID = lookupIDOrCreateNode(arc.to, nodeNameToIDs, nodes);
            const edgeID = uuidv4();
            edges.push({ id: edgeID, source: sourceID, target: targetID, data: { type: edgeType, objectTypes: arc.label, cardinality: arc.counts }, ...getMarkersForEdge(edgeType, edgeID) })
        // }
        // }
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

    let isObject: ActivityNode['data']['isObject'] = undefined;
    if (typeof node === "object") {
        node = (node as { activity: string }).activity;
    }
    if (node.includes("<init> ")) {
        isObject = "init";
    } else if (node.includes("<exit> ")) {
        isObject = "exit";
    }
    if (false || nodeIDMap[node] == undefined) {
        const id = uuidv4();
        nodes.push({ id: id, type: "activity", position: { x: 0, y: 0 }, data: { isObject, type: node.replace("<init> ", "").replace("<exit> ", "") } })
        nodeIDMap[node] = id;
        return id;
    } else {
        return nodeIDMap[node];
    }
}

