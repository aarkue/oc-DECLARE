import { CustomEdge } from "@/edges/types";
import { ActivityNode } from "@/nodes/types";
import { useEdges, useReactFlow } from "@xyflow/react";
import { useRef, useState } from "react";
import init, { get_edge_violation_percentage, load_ocel, unload_ocel } from "../../../crates/backend-wasm/pkg/backend_wasm";
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
                const ocelJSON = await inputRef.current?.files[0].text()
                load_ocel(ocelJSON);
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
            <Button disabled={selectedEdges.length !== 1} onClick={async () => {
                const e = selectedEdges[0];
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
                console.log({ violationPercentage: 100 * violations[1] / violations[0] });
            }}>
                Evaluate
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