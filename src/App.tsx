import {
  Background,
  Controls,
  Panel,
  ReactFlow,
  ReactFlowInstance,
  useEdgesState,
  type OnConnect
} from '@xyflow/react';
import { useCallback, useEffect, useRef } from 'react';


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
  // const instance = useReactFlow();
  console.log("Re-render of App")
  const onConnect = useCallback<OnConnect>((connection) => {
    console.log(connection);
    return flowRef.current?.setEdges((edges) => {
      const edgeType: EdgeType = "ef";
      const newEdge: CustomEdge = {
        ...connection,
        id: Math.random() + connection.source + "@" + connection.sourceHandle + "-" + connection.target + "@" + connection.targetHandle,
        type: "default",
        data: { type: edgeType, objectTypes: ["order"] },
        ...getMarkersForEdge(edgeType)
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
        ev.clipboardData.setData("application/json+oc-declare-flow", data);
      }
      toast("Copied selection!", { icon: <ClipboardCopy /> });
    }

    function addPastedData(
      nodes: ActivityNode[],
      edges: CustomEdge[],
    ) {
      const idPrefix = Date.now() + `-p-${Math.floor(Math.random() * 100)}-`;
      const instance = flowRef.current!;
      const nodeRect = nodes.length > 0 ? nodes[0].position : { x: 0, y: 0 };
      const { x, y } = instance.screenToFlowPosition(mousePos.current);
      const firstNodeSize = { width: 100, minHeight: 50 };
      const xOffset = x - nodeRect.x - firstNodeSize.width / 2;
      const yOffset = y - nodeRect.y - firstNodeSize.minHeight / 2;
      // Mutate nodes to update position and IDs (+ select them)
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
              // sourceHandle: idPrefix + e.sourceHandle,
              // targetHandle: idPrefix + e.targetHandle,
              selected: true,
              data: e.data,
              ...getMarkersForEdge(e.data!.type)
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
    setEdges((edges) => edges.map(e => ({ ...e, ...getMarkersForEdge(e.data!.type) })))
  }, [setEdges])
  const contextMenuTriggerRef = useRef<HTMLButtonElement>(null);
  return (
    <>
      <Toaster />
      <ContextMenu>
        <ContextMenuTrigger className='pointer-events-auto hidden' asChild>
          <button ref={contextMenuTriggerRef}></button>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onClick={(ev) => {
            ev.stopPropagation();
            flowRef.current?.addNodes({ id: Date.now() + "-" + Math.random(), type: "activity", data: { type: "pay order" }, position: flowRef.current.screenToFlowPosition({ x: ev.clientX, y: ev.clientY }) })
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
        maxZoom={8}
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
          selectedRef.current = sel as any;
        }}
        fitView
        proOptions={{ hideAttribution: true }}
      >
        <Background className='hide-in-image' />
        <Controls className='hide-in-image' />
        <Panel className='flex gap-x-1 hide-in-image'>
          <button className='bg-white border p-1 hover:bg-gray-100' onClick={() => {
            flowRef.current?.addNodes({
              id: Date.now() + "-node",
              position: { x: 0, y: 0 },
              dragHandle: '.drag-handle__custom', data: Math.random() > 0.5 ? { type: "pay order", isObject: false } : { type: "order", isObject: true },
              type: 'activity',
            });
          }}>Add Node</button>

          <button className='bg-white border p-1 hover:bg-gray-100' onClick={() => {
            localStorage.setItem("oc-DECLARE", JSON.stringify(flowRef.current!.toObject()));
          }}>Save</button>

          <button className='bg-white border p-1 hover:bg-gray-100' onClick={() => {
            const flow = loadData();
            if (flow && flowRef.current) {
              const { x = 0, y = 0, zoom = 1 } = flow.viewport;
              flowRef.current.setNodes(flow.nodes || []);
              setEdges(flow.edges || []);
              flowRef.current.setViewport({ x, y, zoom });
            }
          }}>Restore</button>
          <button className='bg-white border p-1 hover:bg-gray-100 disabled:bg-gray-500 disabled:hover:bg-gray-500' onClick={(ev) => {
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
                  canvasHeight: viewPort.clientHeight * scaleFactor * 2,
                  canvasWidth: viewPort.clientWidth * scaleFactor * 2,
                  filter: (node) => {
                    return node.classList === undefined ||
                      !node.classList.contains("hide-in-image")
                  }
                })
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
          }}>Download Image</button>
          <BackendButton/>
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
        </svg></div>
    </>
  );
}
