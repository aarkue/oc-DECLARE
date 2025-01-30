import { CustomEdge } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { useEdges, useReactFlow } from "@xyflow/react";
import { useRef, useState } from "react";
import init, { get_edge_violation_percentage, load_ocel, unload_ocel } from "../../../crates/backend-wasm/pkg/backend_wasm";
import type { OCDeclareArc } from "../../../crates/shared/bindings/OCDeclareArc";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
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
                const ocelJSON = await inputRef.current?.files[0].text()
                load_ocel(ocelJSON);
                // console.log("Got ocel pointer: " + ocelRef.current);
                setStatus("ocel-loaded");
            }
        }}>Load</Button>}
        {status === "ocel-loaded" && <Button variant="destructive" onClick={() => {
            unload_ocel();
            setStatus("initial");
        }} >
            Unload</Button>}
        {status === "ocel-loaded" &&
            <Button disabled={selectedEdges.length !== 1} onClick={async () => {
                const e = selectedEdges[0];
                const x: OCDeclareArc = {
                    from: { type: "Activity", activity: flow.getNode(e.source)!.data.type },
                    to: { type: "Activity", activity: flow.getNode(e.target)!.data.type },
                    arc_type: "ASS",
                    label: {
                        each: e.data!.objectTypes!.map(ot => {
                            if (typeof ot === 'string') {
                                return { type: "Simple", object_type: ot }
                            } else {
                                return { type: "O2O", first: ot[0], second: ot[1], reversed: false }
                            }
                        }),
                        any: [],
                        all: []
                    }
                };
                const violationPercentage = get_edge_violation_percentage(JSON.stringify(x));
                console.log({ violationPercentage });
            }}>
                Evaluate
            </Button>
        }
    </>
}