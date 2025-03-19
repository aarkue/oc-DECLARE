
import { OCDeclareArc } from "crates/shared/bindings/OCDeclareArc";
import init, { discover_oc_declare_constraints, get_all_edge_violation_percentage_perf, get_edge_as_template_text, get_ot_act_involvements, initThreadPool, load_ocel_json, load_ocel_xml, unload_ocel } from "../../crates/backend-wasm/pkg/backend_wasm";

// listen for messages from UI thread
onmessage = function (e: MessageEvent<{ type: "load-ocel", file: File } | { type: "unload-ocel" } | { type: "discover" } | { type: "evaluate-edges", edges: OCDeclareArc[], edgeIDs: string[] } | { type: "" }>) {
    if (e.data.type === "load-ocel") {
        const data = e.data;
        // read contents of file
        const reader = new FileReader();
        reader.onload = async () => {
            const x = new Uint8Array(reader.result as ArrayBuffer);

            await init();
            try {
                await await initThreadPool(Math.max(1, Math.round(0.25 * navigator.hardwareConcurrency)));
            } catch (e) {
                console.log("Thread pool error: ", e);
            }
            data.file.name.endsWith(".json") ? load_ocel_json(x) : load_ocel_xml(x)
            // console.log("Got ocel pointer: " + ocelRef.current);
            // setStatus("ocel-loaded");
            const otActInvolvement = JSON.parse(get_ot_act_involvements());

            this.postMessage({ type: "ocel-loaded", info: otActInvolvement });
        };
        reader.readAsArrayBuffer(e.data.file);
    } else if (e.data.type === "unload-ocel") {
        unload_ocel();
        this.postMessage({ type: "ocel-unloaded" });
    } else if (e.data.type === "discover") {

        let now = Date.now();
        const res = discover_oc_declare_constraints(0.2);
        const discoveredArcs: OCDeclareArc[] = JSON.parse(res);
        console.log("Discovery took " + ((Date.now() - now) / 1000) + "s");
        this.postMessage({ type: "discovered", discoveredArcs });
    } else if (e.data.type === "evaluate-edges") {
        const edgeJSON = JSON.stringify(e.data.edges);
        console.log(edgeJSON);
        for(const x of e.data.edges){
            console.log(get_edge_as_template_text(JSON.stringify(x)));
        }
        const beginning = Date.now();
        const violFracs = get_all_edge_violation_percentage_perf(edgeJSON);
        console.log("TOTAL Evaluation took " + ((Date.now() - beginning) / 1000) + "s");
        this.postMessage({ type: "edges-evaluated", edgeIDs: e.data.edgeIDs, violFracs })
    }
    else {
        console.warn("Unknown message type: " + e.data.type);
    }
};