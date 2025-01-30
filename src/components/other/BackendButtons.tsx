import { Button } from "../ui/button";
import init, { get_edge_violation_percentage } from "../../../crates/backend-wasm/pkg/backend_wasm"
import { useRef } from "react";
import { useReactFlow } from "@xyflow/react";
import type {OCDeclareArc} from "../../../crates/shared/bindings/OCDeclareArc"
import { ActivityNode } from "@/nodes/types";
import { CustomEdge } from "@/edges/types";
export default function BackendButton() {
    const inputRef = useRef<HTMLInputElement>(null);
    const flow = useReactFlow<ActivityNode, CustomEdge>();
    return <>
        <input type="file" ref={inputRef} />
        <Button onClick={async () => {
            const e = flow.getEdges().find(e => e.selected)!;
            const x: OCDeclareArc = {
                from: {type: "Activity", activity: flow.getNode(e.source)!.data.type},
                to: {type: "Activity", activity: flow.getNode(e.target)!.data.type},
                arc_type: "ASS",
                label: {
                    each: e.data!.objectTypes!.map(ot => {
                        if(typeof ot === 'string'){
                            return {type: "Simple", object_type: ot}
                        }else {
                            return {type: "O2O", first: ot[0], second: ot[1], reversed: false}
                        }
                    }),
                    any: [],
                    all: []
                }
            };
            console.log(x);
            if(inputRef.current?.files?.length) {
                await init();
                const ocelJSON = await inputRef.current?.files[0].text()
                const violationPercentage = get_edge_violation_percentage(ocelJSON,JSON.stringify(x));
                console.log({violationPercentage});
            }
        }}>
            Greet
        </Button></>
}