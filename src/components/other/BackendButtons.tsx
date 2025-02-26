import { CustomEdge } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { useEdges, useReactFlow } from "@xyflow/react";
import { useContext, useRef, useState } from "react";
import init, { discover_oc_declare_constraints, get_all_edge_violation_percentage_perf, get_ot_act_involvements, initThreadPool, load_ocel_json, load_ocel_xml, unload_ocel } from "../../../crates/backend-wasm/pkg/backend_wasm";
import type { OCDeclareArc } from "../../../crates/shared/bindings/OCDeclareArc";

import { OCELInfoContext } from "@/lib/ocel-info";
import { addArcsToFlow, flowEdgeToOCDECLARE } from "@/lib/type-conversions";
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
                    await await initThreadPool(Math.max(1, Math.round(0.25 * navigator.hardwareConcurrency)));
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
                setOcelInfo({});
            }
        }} >
            Unload</Button>}
        {status === "ocel-loaded" &&
            <><Button onClick={async () => {
                const beginning = Date.now();
                const edges = (selectedEdges.length > 0 ? selectedEdges : flow.getEdges());
                const edgeJSON = JSON.stringify(edges.map(e => flowEdgeToOCDECLARE(e, flow)));
                const violationFracs = get_all_edge_violation_percentage_perf(edgeJSON);
                for (let i = 0; i < edges.length; i++) {
                    flow.updateEdgeData(edges[i].id, { violationInfo: { violationPercentage: 100 * violationFracs[i] } });

                }
                // edges.forEach(e => {
                //     const x = flowEdgeToOCDECLARE(e, flow);
                //     const violFrac = getEdgeViolationPerc(x);
                //     console.log(violFrac)
                // });
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
            <Button onClick={async () => {
                try {
                    let now = Date.now();
                    const res = discover_oc_declare_constraints(0.4);
                    console.log("Discovery took " + ((Date.now() - now) / 1000) + "s");
                    const discoverdArcs: OCDeclareArc[] = JSON.parse(res);
                    addArcsToFlow(discoverdArcs, flow);
                    console.log(discoverdArcs);
                } catch (e) {
                    console.error(e);
                }
            }}>Discover</Button>}
    </>

}
