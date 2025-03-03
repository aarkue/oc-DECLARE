import { CustomEdge } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { useEdges, useReactFlow } from "@xyflow/react";
import { useContext, useEffect, useRef, useState } from "react";
import type { OCDeclareArc } from "../../../crates/shared/bindings/OCDeclareArc";

import { OCELInfo, OCELInfoContext } from "@/lib/ocel-info";
import { addArcsToFlow, flowEdgeToOCDECLARE } from "@/lib/type-conversions";
import { Button } from "../ui/button";
import { Input } from "../ui/input";

import WASMWorker from "../../lib/worker?worker";
const worker = new WASMWorker();
export default function BackendButton() {
    const inputRef = useRef<HTMLInputElement>(null);
    const flow = useReactFlow<ActivityNode, CustomEdge>();
    const selectedEdges = useEdges<CustomEdge>().filter(e => e.selected)
    const [status, setStatus] = useState<"initial" | "ocel-loaded">("initial")
    const { setOcelInfo } = useContext(OCELInfoContext);
// TODO: Extract types & Add error handling
    useEffect(() => {
        const messageListener = (e: MessageEvent<{ type: "ocel-loaded", info: OCELInfo } | { type: "ocel-unloaded" } | { type: "discovered", discoveredArcs: OCDeclareArc[] } | { type: "edges-evaluated", edgeIDs: string[], violFracs: number[] } | { type: "" }>) => {
            if (e.data.type === "ocel-loaded") {
                setOcelInfo(e.data.info);
                setStatus("ocel-loaded");
            } else if (e.data.type === "ocel-unloaded") {
                setStatus("initial");
                setOcelInfo({});
            } else if (e.data.type === "discovered") {
                addArcsToFlow(e.data.discoveredArcs, flow);
                console.log(e.data.discoveredArcs);
            } else if (e.data.type === "edges-evaluated") {
                for (let i = 0; i < e.data.edgeIDs.length; i++) {
                    flow.updateEdgeData(e.data.edgeIDs[i], { violationInfo: { violationPercentage: 100 * e.data.violFracs[i] } });
                }
            } else {
                console.warn("Unknown message type: " + e.data.type);
            }
        };
        worker.addEventListener("message", messageListener);
        return () => {
            worker.removeEventListener("message", messageListener);
        }
    })

    return <>
        {status === "initial" && <Input type="file" ref={inputRef} className="max-w-[7rem]" />}
        {status === "initial" && <Button onClick={async () => {
            if (inputRef.current?.files?.length) {
                // await init();
                // try {
                //     await await initThreadPool(Math.max(1, Math.round(0.25 * navigator.hardwareConcurrency)));
                // } catch (e) {
                //     console.log("Thread pool error: ", e);
                // }
                const file = inputRef.current?.files[0];
                worker.postMessage({ type: "load-ocel", file });
                // // const y = await file.bytes()
                // const ocelFileData = await file.arrayBuffer()
                // const x = new Uint8Array(ocelFileData);
                // console.log(x.length)
                // file.name.endsWith(".json") ? load_ocel_json(x) : load_ocel_xml(x)
                // // console.log("Got ocel pointer: " + ocelRef.current);
                // setStatus("ocel-loaded");
                // const otActInvolvement = JSON.parse(get_ot_act_involvements());
                // console.log(otActInvolvement);
                // setOcelInfo(otActInvolvement);
            }
        }}>Load</Button>}
        {status === "ocel-loaded" && <Button variant="destructive" onClick={() => {
            worker.postMessage({ type: "unload-ocel" })
        }} >
            Unload</Button>}
        {status === "ocel-loaded" &&
            <><Button onClick={async () => {
                const edges = (selectedEdges.length > 0 ? selectedEdges : flow.getEdges());
                const edgeIDs = edges.map(e => e.id);
                const edgesConverted = edges.map(e => flowEdgeToOCDECLARE(e, flow));
                worker.postMessage({type: "evaluate-edges", edges: edgesConverted, edgeIDs})
                // const violationFracs = get_all_edge_violation_percentage_perf(edgeJSON);

                // edges.forEach(e => {
                //     const x = flowEdgeToOCDECLARE(e, flow);
                //     const violFrac = getEdgeViolationPerc(x);
                //     console.log(violFrac)
                // });
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
                    worker.postMessage({ type: "discover" });
                } catch (e) {
                    console.error(e);
                }
            }}>Discover</Button>}
    </>

}
