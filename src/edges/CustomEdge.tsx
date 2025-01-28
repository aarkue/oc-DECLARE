import {
    ContextMenu,
    ContextMenuCheckboxItem,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuPortal,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { EdgeLabelRenderer, EdgeProps, getStraightPath, Position, useEdges, useInternalNode, useReactFlow } from '@xyflow/react';
import clsx from 'clsx';
import { getEdgeParams } from './edge-helpers';

import { ActivityNode } from '@/nodes/types';
import { ContextMenuArrow } from '@radix-ui/react-context-menu';
import { LucideArrowLeftRight, LucideHash, LucideShapes, TrendingUp } from 'lucide-react';
import { ALL_EDGE_TYPES, CustomEdge as CustomEdgeType, getMarkersForEdge } from './types';
const DISTANCE_FACTOR = 10;
const interactionWidth = 20;

export default function CustomEdge({ id, source, target, markerEnd, style, markerStart, selected, data }: EdgeProps<CustomEdgeType>) {
    const sourceNode = useInternalNode(source);
    const targetNode = useInternalNode(target);

    const flow = useReactFlow<ActivityNode, CustomEdgeType>();

    if (!sourceNode || !targetNode) {
        return null;
    }
    const duplicates = useEdges().map((e, i) => ({ e, i })).filter(({ e, }) => (e.source === source && e.target == target) || (e.source === target && e.target === source))
    const ownIndex = duplicates.filter(({ e }) => e.id === id).map(({ i }) => i)[0] ?? 0;
    const numberOfEarlierDuplicates = duplicates.filter(({ i }) => i < ownIndex).length;
    const numberOfLaterDuplicates = duplicates.filter(({ i }) => i > ownIndex).length;
    const { sx, sy, tx, ty, targetPos } = getEdgeParams(sourceNode, targetNode);
    const modifiedPos = {
        sourceX: sx + ((targetPos === Position.Bottom || targetPos === Position.Top) ? (numberOfEarlierDuplicates * DISTANCE_FACTOR + numberOfLaterDuplicates * -DISTANCE_FACTOR) : 0),
        sourceY: sy + ((targetPos === Position.Left || targetPos === Position.Right) ? (numberOfEarlierDuplicates * DISTANCE_FACTOR + numberOfLaterDuplicates * -DISTANCE_FACTOR) : 0),
        targetX: tx + ((targetPos === Position.Bottom || targetPos === Position.Top) ? (numberOfEarlierDuplicates * DISTANCE_FACTOR + numberOfLaterDuplicates * -DISTANCE_FACTOR) : 0),
        targetY: ty + ((targetPos === Position.Left || targetPos === Position.Right) ? (numberOfEarlierDuplicates * DISTANCE_FACTOR + numberOfLaterDuplicates * -DISTANCE_FACTOR) : 0),
    };
    const [edgePath, labelX, labelY] = getStraightPath(modifiedPos);

    let slopeDegree = Math.atan2((modifiedPos.targetY - modifiedPos.sourceY), (modifiedPos.targetX - modifiedPos.sourceX)) * 180 / Math.PI;
    if (Math.abs(slopeDegree) > 90) {
        slopeDegree = slopeDegree - 180;
    }
    return (
        <><path
            id={id}
            className="react-flow__edge-path"
            d={edgePath}
            markerStart={markerStart}
            markerEnd={markerEnd}
            style={style} />
            <ContextMenu>
                <ContextMenuTrigger className='pointer-events-auto' asChild>
                    {/* Right click */}
                    {interactionWidth && (
                        <path
                            d={edgePath}
                            //   fill="none"
                            //   strokeOpacity={0}
                            strokeWidth={interactionWidth}
                            className={clsx("react-flow__edge-interaction stroke-red-400/0 hover:stroke-blue-400/5", selected && "stroke-blue-400/5")}
                        />
                    )}
                </ContextMenuTrigger>
                <ContextMenuContent>

                    <ContextMenuSub>
                        <ContextMenuSubTrigger>
                            <TrendingUp className='size-4 mr-1' /> Edit Edge Type
                        </ContextMenuSubTrigger>
                        <ContextMenuPortal>
                            <ContextMenuSubContent>
                                {ALL_EDGE_TYPES.map((et) => <ContextMenuCheckboxItem checked={data?.type === et} key={et} onClick={() => {
                                    flow.updateEdge(id, { data: { ...data, type: et }, ...getMarkersForEdge(et) })
                                }}>
                                 
                                    {et}
                                </ContextMenuCheckboxItem>)}
                                <ContextMenuArrow />

                            </ContextMenuSubContent>
                        </ContextMenuPortal>
                    </ContextMenuSub>
                    <ContextMenuSeparator />
                    <ContextMenuItem><LucideShapes className='size-4 mr-1' />Edit Object Types</ContextMenuItem>
                    <ContextMenuItem><LucideHash className='size-4 mr-1' /> Edit Cardinality</ContextMenuItem>
                    <ContextMenuItem onClick={() => {
                        flow.updateEdge(id, { source: target, target: source })

                    }}><LucideArrowLeftRight className='size-4 mr-1' /> Switch Direction</ContextMenuItem>
                    <ContextMenuItem className='text-red-600 hover:focus:text-red-500' onClick={() => {
                        flow.deleteElements({ edges: [{ id }] })
                    }}>Remove</ContextMenuItem>
                </ContextMenuContent>
            </ContextMenu>
            <EdgeLabelRenderer>
                <EdgeLabel transform={`translate(${labelX}px,${labelY}px)  translate(-50%, -50%)  rotate(${Math.round(slopeDegree)}deg)   translate(0,-6pt)`} label='Order' />
                {/* <EdgeLabel transform={`translate(-50%, -50%) translate(${modifiedPos.sourceX}px,${modifiedPos.sourceY}px) ${(targetPos === Position.Top) ? "translate(8px,9px)" : targetPos === Position.Left ? "translate(12px,-11px)" : targetPos === Position.Bottom ? "translate(8px,-9px)" : "translate(-11px,-11px)"} `}
                    label={"1"} /> */}
                <EdgeLabel transform={`translate(-50%, -50%) translate(${modifiedPos.targetX}px,${modifiedPos.targetY}px) ${(targetPos === Position.Top) ? "translate(8px,-9px)" : targetPos === Position.Left ? "translate(-12px,-11px)" : targetPos === Position.Bottom ? "translate(8px,9px)" : "translate(11px,-11px)"} `}
                    label={"1"} />
            </EdgeLabelRenderer>
        </>
    );
}


// this is a little helper component to render the actual edge label
function EdgeLabel({ transform, label }: { transform: string; label: string }) {
    return (
        <div
            style={{
                transform,
            }}
            className=" pointer-events-auto  absolute nodrag nopan text-[6pt] text-black! font-normal! bg-white/40 z-9999"
        >
            {label}
        </div>
    );
}
