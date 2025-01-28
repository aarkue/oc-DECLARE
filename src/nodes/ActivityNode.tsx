import { Handle, Position, useConnection, useReactFlow, type NodeProps } from '@xyflow/react';

import clsx from 'clsx';
import { useMemo, useState } from 'react';
import { type ActivityNode } from './types';
import { getRandomStringColor } from '@/lib/random-colors';
const OBJECT_INIT = "<init>";
export function ActivityNode({
  id,
  data,
  selected,
}: NodeProps<ActivityNode>) {
  const [editMode, setEditMode] = useState(false);
  const { setNodes } = useReactFlow();

  const connection = useConnection();

  const isTarget = connection.inProgress && connection.fromNode.id !== id;
  function applyNameEdit(
    ev:
      | React.FocusEvent<HTMLDivElement, Element>
      | React.MouseEvent<HTMLDivElement, MouseEvent>,
  ) {
    const isObject = ev.currentTarget.innerText.includes(OBJECT_INIT);
    const newLabel = ev.currentTarget.innerText.replace("\n", "").replace(OBJECT_INIT + " ", "");
    setEditMode(false);
    setNodes((nodes) => {
      const newNodes = [...nodes];
      newNodes.map((n) => {
        if (n.id === id) {
          n.data = { type: newLabel || "-", isObject };
        }
        return n;
      });
      return newNodes;
    });
  }

  const objectColor = useMemo(() => {
    return data.isObject ? getRandomStringColor(data.type) : undefined;
  }, [data.isObject, data.type])

  return (
    <div className={clsx("border-2 w-[8rem] py-1 px-1 flex items-center justify-center relative min-h-[3.5rem] h-fit bg-white rounded group", !data.isObject && "border-gray-600", selected && "shadow-lg")}
      style={{ borderColor: objectColor }}>
      <div className={clsx("border text-center border-transparent flex items-center min-h-[2rem] w-[calc(100%-1rem)]  drag-handle__custom group-hover:border-dashed group-hover:border-gray-300/50 z-2", connection.inProgress && "pointer-events-none")}>

        <div contentEditable={editMode} className='w-full text-xs pointer-events-auto'
          suppressContentEditableWarning={true}
          onKeyDownCapture={(ev) => {
            if (ev.key === "Enter") {
              ev.preventDefault();
              ev.stopPropagation();
              ev.currentTarget.blur();
            }
          }}
          onMouseDownCapture={(ev) => {
            if (editMode) {
              ev.stopPropagation()
            }
          }}
          onDoubleClick={(ev) => {
            if (editMode) {
              // ev.preventDefault();
              // applyNameEdit(ev);
              ev.stopPropagation();
            } else {
              setEditMode(true);
              const el = ev.currentTarget;
              setTimeout(() => {
                const range = document.createRange();
                const sel = window.getSelection();
                range.selectNodeContents(el);
                if (sel) {
                  sel.removeAllRanges();
                  sel.addRange(range);
                }
              }, 100);
            }
          }}
          onBlur={(ev) => {
            applyNameEdit(ev);
          }}
          spellCheck="false"
          style={{
            overflowWrap: "break-word",
            cursor: editMode ? "text" : undefined,
            overflowY: "hidden",
            // maxWidth: "6rem",
            // minWidth: "4rem",
            // minHeight: "1.5rem",
            display: "block",
            marginInline: "auto",
            textAlign: "center",
            zIndex: 10,
            position: "relative",
          }}
        >
          {(data.isObject ? "<init> " : "") + data.type}
        </div>
      </div>
      {!connection.inProgress && (
        <Handle
          className="customHandle"
          position={Position.Right}
          type="source"
        />
      )}
      {/* We want to disable the target handle, if the connection was started from this node */}
      {(!connection.inProgress || isTarget) && (
        <Handle className="customHandle z-10" position={Position.Left} type="target" isConnectableStart={false} />
      )}
    </div>
  );
}
