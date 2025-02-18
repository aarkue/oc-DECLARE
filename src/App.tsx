import {
  Background,
  Controls,
  Panel,
  ReactFlow,
  ReactFlowInstance,
  useEdgesState,
  type OnConnect
} from '@xyflow/react';
import { useCallback, useEffect, useRef, useState } from 'react';


import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger
} from "@/components/ui/context-menu";
import '@xyflow/react/dist/style.css';
import { toBlob, toSvg } from 'html-to-image';
import { ClipboardCopy, ClipboardPaste } from 'lucide-react';
import toast, { Toaster } from 'react-hot-toast';
import { edgeTypes } from './edges';
import { CustomEdge, EdgeType, getMarkersForEdge } from './edges/types';
import { downloadBlob } from './lib/download-blob';
import { initialNodes, nodeTypes } from './nodes';
import { ActivityNode } from './nodes/types';
import BackendButton from './components/other/BackendButtons';
import { Button } from './components/ui/button';
import { OCELInfo, OCELInfoContext } from './lib/ocel-info';
import { OCDeclareArcLabel } from 'crates/shared/bindings/OCDeclareArcLabel';
import { applyLayoutToNodes } from './lib/automatic-layout';
import { v4 as uuidv4 } from 'uuid';

function loadData() {
  try {
    return JSON.parse(localStorage.getItem("oc-DECLARE") ?? "{}")
  } catch (e) {
    console.log("Failed to import JSON", e);
    return {}
  }
}
export default function App() {
  const flowRef = useRef<ReactFlowInstance<ActivityNode, CustomEdge>>();
  const selectedRef = useRef<{
    nodes: ActivityNode[];
    edges: CustomEdge[];
  }>({ nodes: [], edges: [] });
  const mousePos = useRef<{
    x: number;
    y: number;
  }>({ x: 0, y: 0 });

  const [edges, setEdges, onEdgesChange] = useEdgesState<CustomEdge>([]);
  const [ocelInfo, setOcelInfo] = useState<OCELInfo>({})
  // const instance = useReactFlow();
  console.log("Re-render of App")
  const onConnect = useCallback<OnConnect>((connection) => {
    const source = flowRef.current!.getNode(connection.source)!;
    const target = flowRef.current!.getNode(connection.target)!;
    let objectTypes: OCDeclareArcLabel = { "each": [{ type: "Simple", object_type: "orders" }], any: [], all: [] };
    if (source.data.isObject && !target.data.isObject) {
      objectTypes = { "each": [], any: [{ type: "Simple", object_type: source.data.type }], all: [] };
    } else if (target.data.isObject && !source.data.isObject) {
      objectTypes = { "each": [], any: [{ type: "Simple", object_type: target.data.type }], all: [] };
    } else if (target.data.isObject && source.data.isObject) {
      objectTypes = { "each": [], any: [{ type: "O2O", first: source.data.type, second: target.data.type, reversed: false }], all: [] };
    }
    return flowRef.current?.setEdges((edges) => {
      const edgeType: EdgeType = source.data.isObject || target.data.isObject ? "ass" : "ef";
      const id = Math.random() + connection.source + "@" + connection.sourceHandle + "-" + connection.target + "@" + connection.targetHandle;
      const newEdge: CustomEdge = {
        ...connection,
        type: "default",
        id,
        data: { type: edgeType, objectTypes },
        ...getMarkersForEdge(edgeType, id)
      };
      return [...edges, newEdge]
    })

  }, [])


  function isEditorElementTarget(el: HTMLElement | EventTarget | null) {
    console.log(el);
    if (
      el === document.body ||
      (el !== null && "className" in el && el.className?.includes("react-flow"))
    ) {
      return true
    } else if (el !== null && 'parentElement' in el) {
      return el.parentElement?.parentElement?.className.includes("react-flow") || el.parentElement?.className.includes("react-flow");
    }
  }

  const autoLayout = useCallback(async () => {
    const origEdges = [...flowRef.current!.getEdges()];
    const origNodes = [...flowRef.current!.getNodes()];
    const isSelectionEmpty =
      selectedRef.current.nodes.length <= 1 &&
      selectedRef.current.edges.length <= 1;
    const nodes = isSelectionEmpty
      ? origNodes
      : origNodes.filter((n) => n.selected);
    const edges = (isSelectionEmpty ? origEdges : origEdges).filter(
      (e) =>
        nodes.find((n) => n.id === e.source) !== undefined &&
        nodes.find((n) => n.id === e.target) !== undefined,
    );
    const { x: beforeX, y: beforeY } =
      nodes.length > 0 ? nodes[0].position : { x: 0, y: 0 };
    await applyLayoutToNodes(nodes, edges);
    if (!isSelectionEmpty) {
      const { x: afterX, y: afterY } =
        nodes.length > 0 ? nodes[0].position : { x: 0, y: 0 };
      const diffX = beforeX - afterX;
      const diffY = beforeY - afterY;
      nodes.forEach((n) => {
        n.position.x += diffX;
        n.position.y += diffY;
      });
    }
    flowRef.current!.setNodes(origNodes);
    console.log(isSelectionEmpty);
    if (isSelectionEmpty) {
      setTimeout(() => {
      flowRef.current?.fitView({ duration: 200, padding: 0.2 });
      });
    }
  }, []);

  useEffect(() => {

    function mouseListener(ev: MouseEvent) {
      mousePos.current = { x: ev.x, y: ev.y };
    }


    async function copyListener(ev: ClipboardEvent) {
      if (!isEditorElementTarget(ev.target)) {
        return;
      }
      ev.preventDefault();
      if (ev.clipboardData !== null) {
        const data = JSON.stringify(selectedRef.current);
        console.log({ ...selectedRef.current })
        ev.clipboardData.setData("application/json+oc-declare-flow", data);
        ev.clipboardData.setData("text/plain", data);
      }
      toast("Copied selection!", { icon: <ClipboardCopy /> });
    }

    function addPastedData(
      nodes: ActivityNode[],
      edges: CustomEdge[],
    ) {
      const idPrefix = uuidv4();
      const instance = flowRef.current!;
      const nodeRect = nodes.length > 0 ? nodes[0].position : { x: 0, y: 0 };
      const { x, y } = instance.screenToFlowPosition(mousePos.current);
      const firstNodeSize = { width: 100, minHeight: 50 };
      const xOffset = x - nodeRect.x - firstNodeSize.width / 2;
      const yOffset = y - nodeRect.y - firstNodeSize.minHeight / 2;
      // Mutate nodes to update position and IDs (+ select them)
      console.log({ nodes, edges })
      const newNodes = nodes.map((n) => ({
        id: idPrefix + n.id,
        position: { x: n.position.x + xOffset, y: n.position.y + yOffset },
        selected: true,
        data: n.data,
        type: n.type,
      }));
      // Update nodes
      instance.setNodes((prevNodes) => {
        return [
          // Unselect all existing nodes
          ...prevNodes.map((n) => ({ ...n, selected: false })),
          // ...and add pasted nodes
          ...newNodes,
        ];
      });
      // Update edges
      instance.setEdges((prevEdges) => {
        return [
          // Unselect all exisiting edges
          ...prevEdges.map((e) => ({ ...e, selected: false })),
          // ...and add new pasted edges (mutating the ID, and source/target (handle) + selecting them)
          ...edges
            .map((e) => ({
              id: idPrefix + e.id,
              type: e.type,
              source: idPrefix + e.source,
              target: idPrefix + e.target,
              sourceHandle: e.sourceHandle,
              targetHandle: e.targetHandle,
              selected: true,
              data: e.data,
              ...getMarkersForEdge(e.data!.type, e.id)
            }))
            .filter(
              (e) =>
                newNodes.find((n) => n.id === e.source) !== undefined &&
                newNodes.find((n) => n.id === e.target) !== undefined,
            ),
        ];
      });
    }

    function pasteListener(ev: ClipboardEvent) {
      if (!isEditorElementTarget(ev.target)) {
        return;
      }
      console.log(ev);
      if (ev.clipboardData != null) {
        let pastedNodesAndEdges = ev.clipboardData.getData(
          "application/json+oc-declare-flow",
        );
        if (pastedNodesAndEdges === "") {
          pastedNodesAndEdges = ev.clipboardData.getData("text/plain");

        }
        try {
          const { nodes, edges }: typeof selectedRef.current =
            JSON.parse(pastedNodesAndEdges);
          addPastedData(nodes, edges);
          toast("Pasted selection!", { icon: <ClipboardPaste /> });
        } catch (e) {
          toast("Failed to parse pasted data. Try using Alt+C to copy nodes.");
          console.error("Failed to parse JSON on paste: ", pastedNodesAndEdges);
        }
        ev.preventDefault();
      }
    }
    document.addEventListener("copy", copyListener);
    // document.addEventListener("cut", cutListener);
    document.addEventListener("paste", pasteListener);
    // document.addEventListener("keydown", keyPressListener);
    document.addEventListener("mousemove", mouseListener);
    return () => {
      document.removeEventListener("copy", copyListener);
      // document.removeEventListener("cut", cutListener);
      document.removeEventListener("paste", pasteListener);
      // document.removeEventListener("keydown", keyPressListener);
      document.removeEventListener("mousemove", mouseListener);
    };
  }, [flowRef.current])
  useEffect(() => {
    setEdges((edges) => edges.map(e => ({ ...e, ...getMarkersForEdge(e.data!.type, e.id) })))
  }, [setEdges])
  const contextMenuTriggerRef = useRef<HTMLButtonElement>(null);
  return (
    <>
      <OCELInfoContext.Provider value={{
        ocelInfo: ocelInfo, setOcelInfo: (oi) => {
          setOcelInfo(oi);
        }
      }}>
        <Toaster />
        <ContextMenu>
          <ContextMenuTrigger className='pointer-events-auto hidden' asChild>
            <button ref={contextMenuTriggerRef}></button>
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuItem onClick={(ev) => {
              ev.stopPropagation();
              flowRef.current?.addNodes({ id: uuidv4(), type: "activity", data: { type: "pay order" }, position: flowRef.current.screenToFlowPosition({ x: ev.clientX, y: ev.clientY }) })
              console.log("Add node")
            }}>Add Node</ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
        <div className='outer-flow w-full h-full'><ReactFlow className='react-flow'
          onInit={(i) => flowRef.current = i}
          defaultNodes={initialNodes}
          nodeTypes={nodeTypes}
          edges={edges}
          onEdgesChange={onEdgesChange}
          // onEdgeContextMenu={(ev, edge) => {
          //   ev.preventDefault();
          //   const newType = ALL_EDGE_TYPES[(ALL_EDGE_TYPES.indexOf(edge.data!.type) + 1) % ALL_EDGE_TYPES.length];
          //   flowRef.current?.updateEdge(edge.id, { ...edge, data: { type: newType }, ...getMarkersForEdge(newType) })
          // }}
          // onNodesChange={onNodesChange}
          edgeTypes={edgeTypes}
          // onEdgesChange={onEdgesChange}
          maxZoom={12}
          minZoom={0.3}
          // defaultEdgeOptions={{
          //   type: "default",
          // }}
          onConnect={onConnect}
          onContextMenu={(ev) => {
            if (!ev.isDefaultPrevented() && contextMenuTriggerRef.current) {
              contextMenuTriggerRef.current.dispatchEvent(new MouseEvent("contextmenu", {
                bubbles: true,
                clientX: ev.clientX,
                clientY: ev.clientY,
              }),);
            }
            ev.preventDefault()
          }}
          onSelectionChange={(sel) => {
            // console.log(sel);
            const addedEdges: Set<string> = new Set();
            for (const n of sel.nodes) {
              for (const n2 of sel.nodes) {
                flowRef.current?.getEdges().filter(e => e.source === n.id && e.target === n2.id && sel.edges.find(e2 => e2.id === e.id) == null).map(e => e.id).forEach(e => addedEdges.add(e))
              }
            }
            if (addedEdges.size > 0) {
              flowRef.current?.setEdges(edges => [...edges].map(e => ({ ...e, selected: e.selected || addedEdges.has(e.id) })))
            }
            selectedRef.current = sel as any;
          }}
          // fitView
          proOptions={{ hideAttribution: true }}
        >
          <Background className='hide-in-image' />
          <Controls className='hide-in-image' />
          <Panel className='flex gap-x-1 hide-in-image'>
            <Button variant="outline" onClick={() => {
              flowRef.current?.addNodes({
                id: uuidv4(),
                position: { x: 0, y: 0 },
                dragHandle: '.drag-handle__custom', data: Math.random() > 0.5 ? { type: "pay order", isObject: false } : { type: "orders", isObject: true },
                type: 'activity',
              });
            }}>Add Node</Button>

            <Button variant="outline" onClick={() => {
              localStorage.setItem("oc-DECLARE", JSON.stringify(flowRef.current!.toObject()));
            }}>Save</Button>

            <Button variant="outline" onClick={() => {
              const flow = loadData();
              if (flow && flowRef.current) {
                const { x = 0, y = 0, zoom = 1 } = flow.viewport;
                flowRef.current.setNodes(flow.nodes || []);
                setEdges(flow.edges || []);
                flowRef.current.setViewport({ x, y, zoom });
              }
            }}>Restore</Button>
            <Button variant="outline" onClick={(ev) => {
              const button = ev.currentTarget;
              button.disabled = true;
              const scaleFactor = 2.0;
              const viewPort = document.querySelector(
                ".outer-flow",
              ) as HTMLElement;
              const useSVG = ev.shiftKey;
              requestAnimationFrame(() => {
                requestAnimationFrame(() => {

                  void (useSVG ? toSvg : toBlob)(viewPort, {
                    canvasHeight: viewPort.clientHeight * scaleFactor * 1,
                    canvasWidth: viewPort.clientWidth * scaleFactor * 1,
                    filter: (node) => {
                      return node.classList === undefined ||
                        !node.classList.contains("hide-in-image")
                    }
                  }).catch(e => console.error("Failed to get image:",e))
                    .then(async (dataURLOrBlob) => {
                      let blob = dataURLOrBlob;
                      if (typeof blob === 'string') {
                        blob = await (await fetch(blob)).blob()
                      }
                      downloadBlob(blob as Blob, "oc-DECLARE" + (useSVG ? ".svg" : ".png"))
                    }).finally(() =>
                      button.disabled = false);
                })
              })
            }}>Download Image</Button>
            <BackendButton />
            <Button onClick={() => autoLayout()}>Layout</Button>
          </Panel>
        </ReactFlow><svg width="0" height="0">
            <defs>
              <marker
                className="react-flow__arrowhead"
                id="dot-marker"
                markerWidth="10"
                markerHeight="10"
                viewBox="-20 -20 40 40"
                orient="auto"
                refX="0"
                refY="0"
              >
                <circle cx="0" cy="0" r="10" fill="var(--arrow-primary,black)" />
              </marker>
              <marker
                className="react-flow__arrowhead"
                id="double-arrow-marker"
                markerWidth="10"
                markerHeight="10"
                viewBox="-20 -20 40 40"
                orient="auto"
                refX="17.3"
                refY="10"
              >
                <path d="M-16,0 L4,10 L-16,20 Z" fill="var(--arrow-primary,black)" />
                <path d="M0,0 L20,9.5 L20,10 L20,10.5 L0,20 Z " fill="var(--arrow-primary,black)" />
              </marker>
              <marker
                className="react-flow__arrowhead"
                id="single-arrow-marker"
                markerWidth="10"
                markerHeight="10"
                viewBox="-20 -20 40 40"
                orient="auto"
                refX="16.9"
                refY="10"
              >

                {/* Directly: */}
                {/* <path d="M13.5,0 L13.5,20 L16.5,20 L16.5,0 Z " fill="var(--arrow-primary,black)" /> */}
                <path d="M0,0 L20,9.5 L20,10 L20,10.5 L0,20 Z " fill="var(--arrow-primary,black)" />
              </marker>
              <marker
                className="react-flow__arrowhead"
                id="single-arrow-direct-marker"
                markerWidth="10"
                markerHeight="10"
                viewBox="-20 -20 40 40"
                orient="auto"
                refX="16.9"
                refY="10"
              >

                {/* Directly: */}
                <path d="M13.5,0 L13.5,20 L16.5,20 L16.5,0 Z " fill="var(--arrow-primary,black)" />
                <path d="M0,0 L20,9.5 L20,10 L20,10.5 L0,20 Z " fill="var(--arrow-primary,black)" />
              </marker>
              <marker
                className="react-flow__arrowhead"
                id="single-not-arrow-marker"
                markerWidth="10"
                markerHeight="10"
                viewBox="-20 -20 40 40"
                orient="auto"
                refX="16.9"
                refY="10"
              >
                <path d="M-15,0 L-13,20 L-10,20 L-12,0 Z" fill="var(--arrow-primary,black)" />
                <path d="M-10,0 L-8,20 L-5,20 L-7,0 Z" fill="var(--arrow-primary,black)" />
                <path d="M0,0 L20,9.5 L20,10 L20,10.5 L0,20 Z " fill="var(--arrow-primary,black)" />
              </marker>
              <marker
                className="react-flow__arrowhead"
                id="single-not-arrow-direct-marker"
                markerWidth="10"
                markerHeight="10"
                viewBox="-20 -20 40 40"
                orient="auto"
                refX="16.9"
                refY="10"
              >
                <path d="M13.5,0 L13.5,20 L16.5,20 L16.5,0 Z " fill="var(--arrow-primary,black)" />
                <path d="M-15,0 L-13,20 L-10,20 L-12,0 Z" fill="var(--arrow-primary,black)" />
                <path d="M-10,0 L-8,20 L-5,20 L-7,0 Z" fill="var(--arrow-primary,black)" />
                <path d="M0,0 L20,9.5 L20,10 L20,10.5 L0,20 Z " fill="var(--arrow-primary,black)" />
              </marker>
              <marker
                className="react-flow__arrowhead"
                id="single-not-arrow-marker-rev"
                markerWidth="40"
                markerHeight="40"
                viewBox="-80 -80 160 160"
                orient="auto"
                refX="0"
              >
                <circle cx="0" cy="0" r="10" fill="var(--arrow-primary,black)" />
                <g transform="rotate(180,0,0) translate(-26, -10)">
                  <path d="M-15,0 L-13,20 L-10,20 L-12,0 Z" fill="var(--arrow-primary,black)" />
                  <path d="M-10,0 L-8,20 L-5,20 L-7,0 Z" fill="var(--arrow-primary,black)" />
                  <path d="M0,0 L20,9.5 L20,10 L20,10.5 L0,20 Z " fill="var(--arrow-primary,black)" />
                </g>
              </marker>
              <marker
                className="react-flow__arrowhead"
                id="single-arrow-marker-rev"
                markerWidth="40"
                markerHeight="40"
                viewBox="-80 -80 160 160"
                orient="auto"
                refX="0"
              >
                <circle cx="0" cy="0" r="10" fill="var(--arrow-primary,black)" />
                <g transform="rotate(180,0,0) translate(-26, -10)">
                  <path d="M0,0 L20,9.5 L20,10 L20,10.5 L0,20 Z " fill="var(--arrow-primary,black)" />
                </g>
              </marker>
            </defs>
          </svg></div></OCELInfoContext.Provider>
    </>
  );
}
