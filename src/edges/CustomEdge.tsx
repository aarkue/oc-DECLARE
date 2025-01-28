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
import { getRandomStringColor } from "@/lib/random-colors";
import React, { useMemo } from "react";
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

    const slopeRad = Math.atan2((modifiedPos.targetY - modifiedPos.sourceY), (modifiedPos.targetX - modifiedPos.sourceX));
    let slopeDegree = slopeRad * 180 / Math.PI;
    if (Math.abs(slopeDegree) > 90) {
        slopeDegree = slopeDegree - 180;
    }
    const allInvolvedObjectTypesWithColor = useMemo(() => [...new Set(data?.objectTypes?.flatMap(ot => {
        if (typeof ot === "object") {
            return ot
        } else {
            return [ot]
        }
    }) ?? [])].map(t => ({ type: t, color: getRandomStringColor(t) })), [data?.objectTypes]);
    // const objectTypeColors = useMemo(() => {
    //     return allInvolvedObjectTypes.map((ot) => getRandomStringColor(ot))
    // },[allInvolvedObjectTypes]);
    const gradientID = `edge-${id}-gradient`;

    let tDir: Position = Position.Left;
    if(slopeRad > -2.75 && slopeRad <= -0.415){
        tDir = Position.Top
    }else if(slopeRad > -0.415 && slopeRad <= 0.4){
        tDir = Position.Right
    }else if(slopeRad > 0.4 && slopeRad < 2.75) {
        tDir = Position.Bottom
    }
    // const targetLeft = Math.abs(slopeRad) >= 2.75;
    // const targetTop = slopeRad > -2.75 && slopeRad <= -0.415; 
    // const targetRight = slopeRad > -0.415 && slopeRad <= 0.4;
    // const targetBottom = slopeRad > 0.4 && slopeRad < 2.75;
    console.log(tDir)
    const invertGradient = (tDir === Position.Top || tDir === Position.Left);
    const correctedGradient = [...allInvolvedObjectTypesWithColor];
    if(invertGradient) {
        correctedGradient.reverse()
    }
    return (
        <>
            <defs>
                <linearGradient id={gradientID}
                gradientTransform={(tDir === Position.Top || tDir === Position.Bottom) ? "rotate(90)" : ""}
                // gradientTransform={(tDir === Position.Top) ? modifiedPos.sourceX <= modifiedPos.targetX ?  'rotate(0)' : ' translate(-0.5,1)' :  (tDir === Position.Bottom ? 'rotate(90)' : (tDir === Position.Left ? 'scale(-1,1)' : 'rotate(90)'))}
                >
                    {correctedGradient.map((t, i) => <stop key={t.type} offset={`${Math.round(100 * (i / (correctedGradient.length - 1)))}%`} stopColor={t.color} />)}
                    {/* <stop offset="0%" stopColor="red"/>
                <stop offset="100%" stopColor="purple"  /> */}
                </linearGradient>
            </defs>
            <path
                id={id}
                className="react-flow__edge-path"
                d={edgePath}
                markerStart={markerStart}
                markerEnd={markerEnd}
                style={{
                    ...style,
                    // "--arrow-primary": getRandomStringColor(data?.objectTypes ? (typeof data?.objectTypes[0] === "object" ? data.objectTypes[0][1] : data.objectTypes[0] as string) : ""),
                    // "--arrow-primary": 'linear-gradient(to right, red, purple)',
                    stroke: `url(#${gradientID})`
                } as React.CSSProperties
                }
            />
            <ContextMenu>
                <ContextMenuTrigger className='pointer-events-auto' asChild onContextMenu={(ev)=> {
                    ev.stopPropagation();
                }}>
                    {/* Right click */}
                    {interactionWidth && (
                        <path
                            d={edgePath}
                            fill="none"
                            //   strokeOpacity={0}
                            strokeWidth={interactionWidth}
                            className={clsx("react-flow__edge-interaction stroke-transparent hover:stroke-gray-400/5", selected && "!stroke-gray-400/10 hover:!stroke-gray-400/15")}
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
                                {ALL_EDGE_TYPES.map((et) => <ContextMenuCheckboxItem checked={data?.type === et} key={et} onClick={(ev) => {
                                    ev.stopPropagation();
                                    flow.updateEdge(id, { data: { ...data, type: et }, ...getMarkersForEdge(et) })
                                }}>

                                    {et}
                                </ContextMenuCheckboxItem>)}
                                <ContextMenuArrow />

                            </ContextMenuSubContent>
                        </ContextMenuPortal>
                    </ContextMenuSub>
                    <ContextMenuSeparator />
                    <ContextMenuItem onClick={(ev) => {
                        ev.stopPropagation();
                        const newObjectType = prompt("Please enter the object types.") ?? "";
                        if (newObjectType === "") {
                            return;
                        }
                        const res = newObjectType.split(", ").flatMap(s => s.split(",")).map(s => {
                            const splitO2O = s.split("~");
                            if (splitO2O.length == 1) {
                                return splitO2O[0]
                            } else {
                                return [splitO2O[0], splitO2O[1]] as const satisfies [string, string];
                            }
                        })
                        flow.updateEdgeData(id, { objectTypes: res })
                    }}><LucideShapes className='size-4 mr-1' />Edit Object Types</ContextMenuItem>
                    <ContextMenuItem onClick={(ev) => {
                        ev.stopPropagation();
                        const n = parseInt(prompt("Enter a new cardinality.") ?? "");
                        if (!isNaN(n)) {
                            flow.updateEdgeData(id, { cardinality: [n, n] })
                        } else {
                            flow.updateEdgeData(id, { cardinality: undefined })
                        }
                    }}><LucideHash className='size-4 mr-1' /> Edit Cardinality</ContextMenuItem>
                    <ContextMenuItem onClick={(ev) => {
                        ev.stopPropagation();
                        flow.updateEdge(id, { source: target, target: source })

                    }}><LucideArrowLeftRight className='size-4 mr-1' /> Switch Direction</ContextMenuItem>
                    <ContextMenuItem className='text-red-600 hover:focus:text-red-500' onClick={(ev) => {
                        ev.stopPropagation();
                        flow.deleteElements({ edges: [{ id }] })
                    }}>Remove</ContextMenuItem>
                </ContextMenuContent>
            </ContextMenu>
            <EdgeLabelRenderer>
                <EdgeLabel transform={`translate(${labelX}px,${labelY}px)  translate(-50%, -50%)  rotate(${Math.round(slopeDegree)}deg)   translate(0,-6pt)`} label={data?.objectTypes?.map((ot) => {
                    if (typeof ot === 'object') {
                        return `${ot[0]}~${ot[1]}`
                    } else {
                        return ot;
                    }
                }).join(", ") ?? "-"} />
                {/* <EdgeLabel transform={`translate(-50%, -50%) translate(${modifiedPos.sourceX}px,${modifiedPos.sourceY}px) ${(targetPos === Position.Top) ? "translate(8px,9px)" : targetPos === Position.Left ? "translate(12px,-11px)" : targetPos === Position.Bottom ? "translate(8px,-9px)" : "translate(-11px,-11px)"} `}
                    label={"1"} /> */}
                <EdgeLabel transform={`translate(-50%, -50%) translate(${modifiedPos.targetX}px,${modifiedPos.targetY}px) ${(targetPos === Position.Top) ? "translate(8px,-9px)" : targetPos === Position.Left ? "translate(-12px,-11px)" : targetPos === Position.Bottom ? "translate(8px,9px)" : "translate(11px,-11px)"} `}
                    label={data?.cardinality ? data?.cardinality[0]?.toString() ?? "" : ""} />
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
            className=" absolute nodrag nopan text-[6pt] text-black! font-normal! bg-white/40 z-9999"
        >
            {label}
        </div>
    );
}
