import { EdgeProps, getStraightPath, Position, useEdges, useInternalNode } from '@xyflow/react';
import { getEdgeParams } from './edge-helpers';

const DISTANCE_FACTOR = 9;

export default function CustomEdge({ id, source, target, markerEnd, style, markerStart }: EdgeProps) {
    const sourceNode = useInternalNode(source);
    const targetNode = useInternalNode(target);

    if (!sourceNode || !targetNode) {
        return null;
    }
    const duplicates = useEdges().map((e, i) => ({ e, i })).filter(({ e, }) => (e.source === source && e.target == target) || (e.source === target && e.target === source))
    const ownIndex = duplicates.filter(({ e }) => e.id === id).map(({ i }) => i)[0] ?? 0;
    const numberOfEarlierDuplicates = duplicates.filter(({ i }) => i < ownIndex).length;
    const numberOfLaterDuplicates = duplicates.filter(({ i }) => i > ownIndex).length;
    const { sx, sy, tx, ty, targetPos } = getEdgeParams(sourceNode, targetNode);
    const [edgePath] = getStraightPath({
        sourceX: sx + ((targetPos === Position.Bottom || targetPos === Position.Top) ? (numberOfEarlierDuplicates * DISTANCE_FACTOR + numberOfLaterDuplicates * -DISTANCE_FACTOR) : 0),
        sourceY: sy + ((targetPos === Position.Left || targetPos === Position.Right) ? (numberOfEarlierDuplicates * DISTANCE_FACTOR + numberOfLaterDuplicates * -DISTANCE_FACTOR) : 0),
        targetX: tx + ((targetPos === Position.Bottom || targetPos === Position.Top) ? (numberOfEarlierDuplicates * DISTANCE_FACTOR + numberOfLaterDuplicates * -DISTANCE_FACTOR) : 0),
        targetY: ty + ((targetPos === Position.Left || targetPos === Position.Right) ? (numberOfEarlierDuplicates * DISTANCE_FACTOR + numberOfLaterDuplicates * -DISTANCE_FACTOR) : 0),
    });


    return (
        <path
            id={id}
            className="react-flow__edge-path"
            d={edgePath}
            markerStart={markerStart}
            markerEnd={markerEnd}
            style={style}
        />
    );
}
