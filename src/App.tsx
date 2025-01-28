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


import '@xyflow/react/dist/style.css';

import { toBlob, toSvg } from 'html-to-image';
import { edgeTypes } from './edges';
import { CustomEdge, EdgeType, getMarkersForEdge } from './edges/types';
import { downloadBlob } from './lib/download-blob';
import { initialNodes, nodeTypes } from './nodes';
import { AppNode } from './nodes/types';
function loadData() {
  try {
    return JSON.parse(localStorage.getItem("oc-DECLARE") ?? "{}")
  } catch (e) {
    console.log("Failed to import JSON", e);
    return {}
  }
}
export default function App() {
  const flowRef = useRef<ReactFlowInstance<AppNode, CustomEdge>>();

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
        data: { type: edgeType, objectTypes: ["order"]  },
        ...getMarkersForEdge(edgeType)
        // markerStart: 'dot-marker',
        // style: { stroke: "var(--arrow-primary)", strokeWidth: 2, strokeDasharray: isAssociationEdge ? "5 5" : undefined },
        // markerEnd:  isAssociationEdge ? undefined :  (Math.random() > 0.5 ? "single-arrow-marker": "double-arrow-marker")
      };
      return [...edges, newEdge]
      // return addEdge({ ...connection,id: Math.random() + connection.source+"@" +connection.sourceHandle+"-" + connection.target + "@" + connection.targetHandle, type: "default", data: { type: "test" }, markerStart: 'dot-marker', markerEnd: { type: MarkerType.ArrowClosed, width: 18, height: 18 } }, edges)
    })

  }, [])

  useEffect(() => {
    setEdges((edges) => edges.map(e => ({ ...e, ...getMarkersForEdge(e.data!.type) })))
  }, [setEdges])
  return (
    <div className='outer-flow w-full h-full'><ReactFlow
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
      onContextMenu={(ev) => ev.preventDefault()}
      // isValidConnection={(c) => {
      //   // const source = flowRef.current?.getNode(c.source);
      //   // const target = flowRef.current?.getNode(c.target);
      //   // const sourceHandleIndex = parseInt(c.sourceHandle?.split("-")[1] ?? "");
      //   // const targetHandleIndex = parseInt(c.targetHandle?.split("-")[1] ?? "");
      //   // if (source === undefined || target === undefined || isNaN(sourceHandleIndex) || isNaN(targetHandleIndex)) {
      //   //   return false;
      //   // }
      //   // const output = getOutputsForStep(source.data.stepType)[sourceHandleIndex];
      //   // const input = getInputForStep(target.data.stepType)[targetHandleIndex];
      //   // return input.type === output.type;
      //   return true;
      // } }
      fitView
      proOptions={{hideAttribution: true}}
    >
      <Background  className='hide-in-image'/>
      <Controls  className='hide-in-image'/>
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
                if(typeof blob === 'string'){
                  blob = await (await fetch(blob)).blob()
                }
                downloadBlob(blob as Blob,"oc-DECLARE" +( useSVG ? ".svg" : ".png"))
              }).finally(() => 
                button.disabled = false);
              })})
        }}>Download Image</button>
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
            refX="16.8"
            refY="10"
          >
            <path d="M-16,0 L4,10 L-16,20 Z" fill="var(--arrow-primary,black)" />
            <path d="M0,0 L20,10 L0,20 Z" fill="var(--arrow-primary,black)" />
          </marker>
          <marker
            className="react-flow__arrowhead"
            id="single-arrow-marker"
            markerWidth="10"
            markerHeight="10"
            viewBox="-20 -20 40 40"
            orient="auto"
            refX="16"
            refY="10"
          >
            <path d="M0,0 L20,10 L0,20 Z" fill="var(--arrow-primary,black)" />
          </marker>
          <marker
            className="react-flow__arrowhead"
            id="single-not-arrow-marker"
            markerWidth="10"
            markerHeight="10"
            viewBox="-20 -20 40 40"
            orient="auto"
            refX="16"
            refY="10"
          >
            <path d="M-15,0 L-13,20 L-10,20 L-12,0 Z" fill="var(--arrow-primary,black)" />
            <path d="M-10,0 L-8,20 L-5,20 L-7,0 Z" fill="var(--arrow-primary,black)" />
            <path d="M0,0 L20,10 L0,20 Z" fill="var(--arrow-primary,black)" />
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
              <path d="M0,0 L20,10 L0,20 Z" fill="var(--arrow-primary,black)" />
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
              <path d="M0,0 L20,10 L0,20 Z" fill="var(--arrow-primary,black)" />
            </g>
          </marker>
        </defs>
      </svg></div>
  );
}
