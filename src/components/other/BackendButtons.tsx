import { CustomEdge, EdgeType, getMarkersForEdge } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { ReactFlowInstance, useEdges, useReactFlow } from "@xyflow/react";
import { useContext, useRef, useState } from "react";
import { v4 as uuidv4 } from 'uuid';
import init, { discover_oc_declare_constraints, get_edge_violation_percentage_perf, get_ot_act_involvements, initThreadPool, load_ocel_json, load_ocel_xml, unload_ocel } from "../../../crates/backend-wasm/pkg/backend_wasm";
import type { OCDeclareArc } from "../../../crates/shared/bindings/OCDeclareArc";

import { OCELInfoContext } from "@/lib/ocel-info";
import { OCDeclareArcType } from "crates/shared/bindings/OCDeclareArcType";
import { OCDeclareNode } from "crates/shared/bindings/OCDeclareNode";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
export default function BackendButton() {
    const inputRef = useRef<HTMLInputElement>(null);
    const flow = useReactFlow<ActivityNode, CustomEdge>();
    const selectedEdges = useEdges<CustomEdge>().filter(e => e.selected)
    const [status, setStatus] = useState<"initial" | "ocel-loaded">("initial")
    const { setOcelInfo } = useContext(OCELInfoContext);
    return <>
        {status === "initial" && <Input type="file" ref={inputRef} className="max-w-[7rem]" />}
        {status === "initial" && <Button onClick={async () => {
            if (inputRef.current?.files?.length) {
                await init();
                try {

                    await await initThreadPool(navigator.hardwareConcurrency);
                } catch (e) {
                    console.log("Thread pool error: ", e);
                }
                const file = inputRef.current?.files[0];
                // const y = await file.bytes()
                const ocelFileData = await file.arrayBuffer()
                const x = new Uint8Array(ocelFileData);
                console.log(x.length)
                file.name.endsWith(".json") ? load_ocel_json(x) : load_ocel_xml(x)
                // console.log("Got ocel pointer: " + ocelRef.current);
                setStatus("ocel-loaded");
                const otActInvolvement = JSON.parse(get_ot_act_involvements());
                console.log(otActInvolvement);
                setOcelInfo(otActInvolvement);
            }
        }}>Load</Button>}
        {status === "ocel-loaded" && <Button variant="destructive" onClick={() => {
            try {

                unload_ocel();
            } finally {
                setStatus("initial");
            }
        }} >
            Unload</Button>}
        {status === "ocel-loaded" &&
            <><Button onClick={async () => {
                const beginning = Date.now();
                (selectedEdges.length > 0 ? selectedEdges : flow.getEdges()).forEach(e => {
                    const [arc_type, counts] = translateArcInfo(e.data!);

                    const x: OCDeclareArc = {
                        from: flow.getNode(e.source)!.data.isObject ? { type: "ObjectInit", object_type: flow.getNode(e.source)!.data.type } : { type: "Activity", activity: flow.getNode(e.source)!.data.type },
                        to: flow.getNode(e.target)!.data.isObject ? { type: "ObjectInit", object_type: flow.getNode(e.target)!.data.type } : { type: "Activity", activity: flow.getNode(e.target)!.data.type },
                        arc_type,
                        counts,
                        label: e.data!.objectTypes
                    };
                    // console.log(x);
                    // console.log(JSON.stringify(x));
                    // const before = Date.now();
                    const xJson = JSON.stringify(x);
                    // const res = get_edge_violation_percentage(xJson);
                    // console.log("Evaluation took " + ((Date.now() - before) / 1000) + "s");
                    // console.log({ res });
                    // const violations: [number, number, [number, ViolationInfo[]][]] = JSON.parse(res);
                    // const violationPercentage = 100 * violations[1] / violations[0];
                    // console.log(violations[2]);
                    const violFrac = get_edge_violation_percentage_perf(xJson);
                    flow.updateEdgeData(e.id, { violationInfo: { violationPercentage: 100 * violFrac } });
                });
                console.log("TOTAL Evaluation took " + ((Date.now() - beginning) / 1000) + "s");
            }}>
                Evaluate {selectedEdges.length === 0 ? "All" : ""}
            </Button>
            </>
        }
        <Button variant="ghost" onClick={() => {
            flow.setEdges(eds => [...eds].map(e => ({ ...e, data: { ...e.data!, violationInfo: undefined } })))
        }}>Reset</Button>

        {status === "ocel-loaded" &&
            <Button onClick={() => {
                try {
                    let now = Date.now();
                    const res = discover_oc_declare_constraints(0.2);
                    console.log("Discovery took " + ((Date.now() - now) / 1000) + "s");
                    const discoverdArcs: OCDeclareArc[] = JSON.parse(res);
                    const nodeNameToIDs: Record<string, string> = {};
                    for (const arc of discoverdArcs) {
                        const sourceID = lookupIDOrCreateNode(arc.from, nodeNameToIDs, flow);
                        const targetID = lookupIDOrCreateNode(arc.to, nodeNameToIDs, flow);
                        const edgeType = translateArcTypeFromRsToTs(arc.arc_type);
                        const edgeID = uuidv4();
                        flow.addEdges({ id: edgeID, source: sourceID, target: targetID, data: { type: edgeType, objectTypes: arc.label, cardinality: arc.counts }, ...getMarkersForEdge(edgeType, edgeID) })
                    }
                    console.log(discoverdArcs);
                } catch (e) {
                    console.error(e);
                }
            }}>Discover</Button>}
    </>
    function translateArcInfo(data: CustomEdge['data']): [OCDeclareArcType, [number | null, number | null]] {
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
        };

    }
}

function lookupIDOrCreateNode(node: OCDeclareNode, nodeIDMap: Record<string, string>, flow: ReactFlowInstance<ActivityNode, CustomEdge>): string {
    let nodeName = node.type === "Activity" ? node.activity : (node.type === "ObjectInit" ? "<init> " + node.object_type : "<exit> " + node.object_type);
    const isObject = node.type !== "Activity" || node.activity.includes("<init> ") || node.activity.includes("<exit> ");
    if (true || isObject || nodeIDMap[nodeName] == undefined) {
        const id = uuidv4();
        flow.addNodes({ id: id, type: "activity", position: { x: 0, y: 0 }, data: { isObject, type: node.type === "Activity" ? node.activity.replace("<init> ", "").replace("<exit> ", "") : node.object_type } })
        nodeIDMap[nodeName] = id;
        return id;
    } else {
        return nodeIDMap[nodeName];
    }
}

function translateArcTypeFromRsToTs(arcType: OCDeclareArcType): EdgeType {
    switch (arcType) {
        case "ASS":
            return "ass"
        case "EF":
            return "ef"
        case "EFREV":
            return "ef-rev"
    }
    return "ass"

}