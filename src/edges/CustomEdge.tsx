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

import { getRandomStringColor } from "@/lib/random-colors";
import { ActivityNode } from '@/nodes/types';
import { ContextMenuArrow } from '@radix-ui/react-context-menu';
import { ArrowLeft, ArrowRight, ChevronLeft, ChevronRight, LucideArrowLeftRight, LucideHash, LucideShapes, LucideXCircle, TrendingUp } from 'lucide-react';
import React, { Fragment, useEffect, useMemo, useState } from "react";
import { ALL_EDGE_TYPES, CustomEdge as CustomEdgeType, getMarkersForEdge } from './types';
const DISTANCE_FACTOR = 10;
const interactionWidth = 20;

function orZero(n: number) {
    if (isNaN(n)) {
        return 0;
    }
    return n;
}
export default function CustomEdge({ id, source, target, markerEnd, style, markerStart, selected, data }: EdgeProps<CustomEdgeType> & { data: { type: string } }) {
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
    const allInvolvedObjectTypesWithColor = useMemo(() => [...new Set([...data.objectTypes.each, ...data.objectTypes.all, ...data?.objectTypes.any].flatMap(ot => {
        if (ot.type === "Simple") {
            return [ot.object_type]
        } else {
            return ot.reversed ? [ot.second, ot.first] : [ot.first, ot.second]
        }
    }) ?? [])].map(t => ({ type: t, color: getRandomStringColor(t) })), [data?.objectTypes]);
    // const objectTypeColors = useMemo(() => {
    //     return allInvolvedObjectTypes.map((ot) => getRandomStringColor(ot))
    // },[allInvolvedObjectTypes]);
    const gradientID = `edge-${id}-gradient`;

    let tDir: Position = Position.Left;
    if (slopeRad > -2.75 && slopeRad <= -0.415) {
        tDir = Position.Top
    } else if (slopeRad > -0.415 && slopeRad <= 0.4) {
        tDir = Position.Right
    } else if (slopeRad > 0.4 && slopeRad < 2.75) {
        tDir = Position.Bottom
    }
    // const targetLeft = Math.abs(slopeRad) >= 2.75;
    // const targetTop = slopeRad > -2.75 && slopeRad <= -0.415; 
    // const targetRight = slopeRad > -0.415 && slopeRad <= 0.4;
    // const targetBottom = slopeRad > 0.4 && slopeRad < 2.75;
    // console.log(tDir)
    const invertGradient = (tDir === Position.Top || tDir === Position.Left);
    const correctedGradient = [...allInvolvedObjectTypesWithColor];
    if (invertGradient) {
        correctedGradient.reverse()
    }

    const [showDialog, setShowDialog] = useState<"ot-label">();
    return (
        <>
            <defs>
                <linearGradient id={gradientID}
                    gradientTransform={(tDir === Position.Top || tDir === Position.Bottom) ? "rotate(90)" : ""}
                // gradientTransform={(tDir === Position.Top) ? modifiedPos.sourceX <= modifiedPos.targetX ?  'rotate(0)' : ' translate(-0.5,1)' :  (tDir === Position.Bottom ? 'rotate(90)' : (tDir === Position.Left ? 'scale(-1,1)' : 'rotate(90)'))}
                >
                    {correctedGradient.map((t, i) => <stop key={t.type} offset={`${orZero(Math.round(100 * (i / (correctedGradient.length - 1))))}%`} stopColor={t.color} />)}
                    {/* <stop offset="0%" stopColor="red"/>
                <stop offset="100%" stopColor="purple"  /> */}
                </linearGradient>
            </defs>
            <path
                id={id}
                className="react-flow__edge-path"
                d={edgePath}
                // stroke-linecap="round"
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
            {showDialog === "ot-label" && <EditEdgeLabelsDialog open={showDialog === "ot-label"} initialValue={data.objectTypes} colors={allInvolvedObjectTypesWithColor} onClose={(value) => {

                setShowDialog(undefined);
                if (value !== undefined) {
                    flow.updateEdgeData(id, { objectTypes: value })
                }
            }} />}
            <ContextMenu>
                <ContextMenuTrigger className='pointer-events-auto' asChild onContextMenu={(ev) => {
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
                        setShowDialog("ot-label");
                        // const newObjectType = prompt("Please enter the object types.") ?? "";
                        // if (newObjectType === "") {
                        //     return;
                        // }
                        // const res = newObjectType.split(", ").flatMap(s => s.split(",")).map(s => {
                        //     const splitO2O = s.split("~");
                        //     if (splitO2O.length == 1) {
                        //         return splitO2O[0]
                        //     } else {
                        //         return [splitO2O[0], splitO2O[1]] as const satisfies [string, string];
                        //     }
                        // })
                        // flow.updateEdgeData(id, { objectTypes: res })
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
                    }}>Delete Edge</ContextMenuItem>
                </ContextMenuContent>
            </ContextMenu>
            <EdgeLabelRenderer>
                <EdgeLabel transform={`translate(${labelX}px,${labelY}px)  translate(-50%, -50%)  rotate(${Math.round(slopeDegree)}deg)   translate(0,-7.5pt)`} label={<span className="text-gray-500 font-medium">
                    {/* {[...data.objectTypes.each.map(ot => ({ ot, type: "each" as const })),
                    ...data.objectTypes.all.map(ot => ({ ot, type: "all" as const })),
                    ...data.objectTypes.any.map(ot => ({ ot, type: "any" as const })),
                    ].map((e, i) => <React.Fragment key={i}>
                        {e.ot.type === 'O2O' && <span><span style={{ color: allInvolvedObjectTypesWithColor.find(x => x.type === (e.ot.type === "O2O" ? e.ot.first : ""))?.color }}>{e.ot.first}</span>~<span style={{ color: allInvolvedObjectTypesWithColor.find(x => x.type === (e.ot.type === "O2O" ? e.ot.first : ""))?.color }}>{e.ot.second}</span></span>}
                        {e.ot.type === "Simple" && <span style={{ color: allInvolvedObjectTypesWithColor.find(x => x.type === (e.ot.type === "Simple" ? e.ot.object_type : ""))?.color }}>{e.ot.object_type}</span>}
                        {(data?.objectTypes.all.length + data.objectTypes.any.length + data.objectTypes.each.length) > i + 1 && <span>,{" "}</span>}
                    </React.Fragment>)
                        // }).join(", ") ?? "-"
                    } */}
                    <div className="gap-x-2 flex">
                        <ShowAllObjectTypeAssociationsOfType type="each" associations={data.objectTypes.each} colors={allInvolvedObjectTypesWithColor} />
                        <ShowAllObjectTypeAssociationsOfType type="all" associations={data.objectTypes.all} colors={allInvolvedObjectTypesWithColor} />
                        <ShowAllObjectTypeAssociationsOfType type="any" associations={data.objectTypes.any} colors={allInvolvedObjectTypesWithColor} />
                    </div>
                    {/* {
                        
                        [...data.objectTypes.each.map(ot => ({ ot, type: "each" as const })),
                        ...data.objectTypes.all.map(ot => ({ ot, type: "all" as const })),
                        ...data.objectTypes.any.map(ot => ({ ot, type: "any" as const }))].map((e, i) =>
                            <ShowObjectTypeAssociation key={i} t={e.ot} colors={allInvolvedObjectTypesWithColor} />
                        )

                    } */}
                </span>
                } />
                {/* <EdgeLabel transform={`translate(-50%, -50%) translate(${modifiedPos.sourceX}px,${modifiedPos.sourceY}px) ${(targetPos === Position.Top) ? "translate(8px,9px)" : targetPos === Position.Left ? "translate(12px,-11px)" : targetPos === Position.Bottom ? "translate(8px,-9px)" : "translate(-11px,-11px)"} `}
                    label={"1"} /> */}
                <EdgeLabel transform={`translate(-50%, -50%) translate(${modifiedPos.targetX}px,${modifiedPos.targetY}px) ${(targetPos === Position.Top) ? "translate(8px,-9px)" : targetPos === Position.Left ? "translate(-12px,-11px)" : targetPos === Position.Bottom ? "translate(8px,9px)" : "translate(11px,-11px)"} `}
                    label={data?.cardinality ? data?.cardinality[0]?.toString() ?? "" : ""} />
            </EdgeLabelRenderer>
        </>
    );
}


// this is a little helper component to render the actual edge label
function EdgeLabel({ transform, label }: { transform: string; label: string | React.ReactNode }) {
    return (
        <div
            style={{
                transform,
            }}
            // text-[10pt] for small demo images
            className=" absolute nodrag nopan text-[8pt] text-black! font-normal!  z-9999"
        >
            {label}
        </div>
    );
}



import { Button } from "@/components/ui/button";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { ObjectTypeAssociation } from "crates/shared/bindings/ObjectTypeAssociation";
import { OCDeclareArcLabel } from "crates/shared/bindings/OCDeclareArcLabel";

function EditEdgeLabelsDialog({ open, initialValue, onClose, colors }: { open: boolean, initialValue: OCDeclareArcLabel, onClose: (newValue?: OCDeclareArcLabel) => unknown, colors?: { type: string, color: string }[] },) {
    const [value, setValue] = useState(initialValue);

    const [addValue, setAddValue] = useState<{ mode: "each" | "all" | "any", t: ObjectTypeAssociation }>({ mode: "each", t: { type: "Simple", object_type: "orders" } })
    useEffect(() => {
        setValue(initialValue);
    }, [initialValue]);

    return <Dialog defaultOpen={open} onOpenChange={(open) => {
        if (!open) {
            onClose(value);
        }
    }}>
        <DialogContent className="min-h-[30rem]">
            <DialogHeader>
                <DialogTitle>Edit Edge Object Type Label</DialogTitle>
                <DialogDescription>
                    This action cannot be undone. This will permanently delete your account
                    and remove your data from our servers.
                </DialogDescription>
                <div className="mt-2 flex flex-col h-full">
                    {(["each", "all", "any"] as const).map(t => <div key={t} className="relative min-h-[4rem]">
                        <div className="flex w-[6rem] justify-between">
                            <h3 className="font-medium text-xl ml-2">{t}</h3>
                        </div>
                        <ul className="ml-6 flex  flex-wrap gap-2">
                            {value[t].map((ot, i) => <li key={i} className="border p-1 rounded relative">
                                <ShowObjectTypeAssociation t={ot} colors={colors} />
                                <LucideXCircle className="absolute size-5 -right-2 -top-2 text-red-400 hover:text-red-600" tabIndex={1} onClick={() => {
                                    setValue((v) => {
                                        const changed = [...v[t]];
                                        changed.splice(i, 1);
                                        const newVal = { ...v, [t]: changed }
                                        return newVal;
                                    })
                                }} />
                            </li>)}
                        </ul>
                    </div>)}
                    <div className="border-t pt-2">
                        <h3 className="font-bold text-xl">Add</h3>
                        <ToggleGroup className="mb-2" type="single" variant="outline" value={addValue.mode} onValueChange={newMode => {
                            setAddValue({ mode: newMode as any, t: addValue.t })
                        }}>
                            <ToggleGroupItem value="each">Each</ToggleGroupItem>
                            <ToggleGroupItem value="all">All</ToggleGroupItem>
                            <ToggleGroupItem value="any">Any</ToggleGroupItem>
                        </ToggleGroup>

                        <Tabs defaultValue="Simple" value={addValue.t.type} onValueChange={(v) => {
                            setAddValue({ mode: addValue.mode, t: (v === "Simple" ? { type: "Simple", object_type: addValue.t.type === "O2O" ? addValue.t['first'] : "" } : { type: "O2O", first: addValue.t.type === "Simple" ? addValue.t.object_type : "", second: "orders", reversed: false }) })
                        }} className="">
                            <TabsList className="w-fit mx-auto block">
                                <TabsTrigger value="Simple">Simple (Direct)</TabsTrigger>
                                <TabsTrigger value="O2O">O2O (Indirect)</TabsTrigger>
                            </TabsList>
                            <TabsContent value="Simple">
                                {addValue.t.type === "Simple" && <>Keep it simple!
                                    <div className="mt-1">
                                        <Input type="text" value={addValue.t.object_type} onChange={(ev) => {
                                            setAddValue({ ...addValue, t: { type: "Simple", object_type: ev.currentTarget.value } })
                                        }} />
                                    </div>
                                </>}
                            </TabsContent>
                            <TabsContent value="O2O">
                                {addValue.t.type === "O2O" && <>Via an object-to-object  relationship!
                                    <div className="flex gap-x-2 mt-1">

                                        <Input type="text" value={addValue.t.first} onChange={(ev) => {
                                            setAddValue({ ...addValue, t: { ...addValue.t as any, first: ev.currentTarget.value } })
                                        }} />
                                        <Button size="sm" variant="secondary" onClick={() => {
                                            setAddValue({ ...addValue, t: { ...addValue.t as any, reversed: !(addValue.t as any).reversed } })
                                        }}>
                                            {!addValue.t.reversed && <ArrowRight />}
                                            {addValue.t.reversed && <ArrowLeft />}
                                        </Button>
                                        <Input type="text" value={addValue.t.second} onChange={(ev) => {
                                            setAddValue({ ...addValue, t: { ...addValue.t as any, second: ev.currentTarget.value } })
                                        }} />
                                    </div>
                                </>}
                            </TabsContent>
                            <Button className="mt-2 ml-auto block" onClick={() => {
                                setValue((v) => {
                                    const changed = [...(v[addValue.mode] ?? []), addValue.t]
                                    const newVal = { ...v, [addValue.mode]: [...new Set(changed)] }
                                    return newVal;
                                })
                            }}>Add</Button>
                        </Tabs>
                    </div>
                </div>
            </DialogHeader>
        </DialogContent>
    </Dialog>
}

function ShowAllObjectTypeAssociationsOfType({ type, associations, colors }: { type: "each" | "all" | "any", associations: ObjectTypeAssociation[], colors?: { type: string, color: string }[] }) {
    if (associations.length === 0) {
        return null;
    }
    return <span>
        {type !== "each" &&
            <>{type.toUpperCase()}(</>
        }
        {associations.map((t, i) => <Fragment key={i}>
            <ShowObjectTypeAssociation t={t} colors={colors} />
            {i < associations.length - 1 && ","}
        </Fragment>)}

        {type !== "each" &&
            <>)</>
        }
    </span>
}
function ShowObjectTypeAssociation({ t, colors }: { t: ObjectTypeAssociation, colors?: { type: string, color: string }[] }) {
    if (t.type === "Simple") {
        return <span style={{ color: colors?.find(x => x.type === t.object_type)?.color }}>{t.object_type}</span>
    }
    return <span><span style={{ color: colors?.find(x => x.type === t.first)?.color }}>
        {t.first}

    </span>
        {/* ~ */}
        {t.reversed && <ChevronLeft className="inline-block size-3 -mx-0.75" />}

        {!t.reversed && <ChevronRight className="inline-block size-3 -mx-0.75" />}
        <span style={{ color: colors?.find(x => x.type === t.second)?.color }}>
            {t.second}
        </span>
    </span>
}