import { CustomEdge } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { useEdges, useReactFlow } from "@xyflow/react";
import { useRef, useState } from "react";
import init, { get_edge_violation_percentage, initThreadPool, load_ocel_json, load_ocel_xml, unload_ocel } from "../../../crates/backend-wasm/pkg/backend_wasm";
import type { OCDeclareArc } from "../../../crates/shared/bindings/OCDeclareArc";
import type { ViolationInfo } from "../../../crates/shared/bindings/ViolationInfo";

import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { OCDeclareArcType } from "crates/shared/bindings/OCDeclareArcType";
export default function BackendButton() {
    const inputRef = useRef<HTMLInputElement>(null);
    const flow = useReactFlow<ActivityNode, CustomEdge>();
    const selectedEdges = useEdges<CustomEdge>().filter(e => e.selected)
    const [status, setStatus] = useState<"initial" | "ocel-loaded">("initial")

    return <>
        {status === "initial" && <Input type="file" ref={inputRef} />}
        {status === "initial" && <Button onClick={async () => {
            if (inputRef.current?.files?.length) {
                await init();
                try{

                    await await initThreadPool(navigator.hardwareConcurrency);
                }catch(e){
                    console.log("Thread pool error: ",e);
                }
                const file =  inputRef.current?.files[0];
                // const y = await file.bytes()
                const ocelFileData = await file.bytes()
                const x = new Uint8Array(ocelFileData);
                console.log(x.length)
                file.name.endsWith(".json") ? load_ocel_json(x) : load_ocel_xml(x)
                // console.log("Got ocel pointer: " + ocelRef.current);
                setStatus("ocel-loaded");
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
            <Button onClick={async () => {
                (selectedEdges.length > 0 ? selectedEdges : flow.getEdges()).forEach(e => {
                    const [arc_type, counts] = translateArcInfo(e.data!);
                    
                    const x: OCDeclareArc = {
                        from: { type: "Activity", activity: flow.getNode(e.source)!.data.type },
                        to: { type: "Activity", activity: flow.getNode(e.target)!.data.type },
                        arc_type,
                        counts,
                        label: e.data!.objectTypes
                    };
                    const before = Date.now()
                    const violations: [number, number, [number, ViolationInfo[]][]] = JSON.parse(get_edge_violation_percentage(JSON.stringify(x)));
                    console.log("Evaluation took " + ((Date.now() - before)/1000) + "s")
                    const violationPercentage = 100 * violations[1] / violations[0] 
                    // console.log(violations);
                    flow.updateEdgeData(e.id,{violationInfo: {violationPercentage}});
                })
            }}>
                Evaluate {selectedEdges.length === 0 ? "All" : ""}
            </Button>
        }
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