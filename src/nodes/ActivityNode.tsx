import { Handle, Position, useConnection, useReactFlow, type NodeProps } from '@xyflow/react';

import clsx from 'clsx';
import { useEffect, useMemo, useRef, useState } from 'react';
import { type ActivityNode } from './types';
import { getRandomStringColor } from '@/lib/random-colors';
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger } from '@/components/ui/context-menu';
const OBJECT_INIT = "<init>";
export function ActivityNode({
  id,
  data,
  selected,
}: NodeProps<ActivityNode>) {
  const [editMode, setEditMode] = useState(false);
  const { setNodes } = useReactFlow();
  const contentEditableDiv = useRef<HTMLDivElement>(null);

  const connection = useConnection();
  const flow = useReactFlow();

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

  useEffect(() => {
    if(editMode && contentEditableDiv.current){
      contentEditableDiv.current.focus();
      const range = document.createRange();
      const sel = window.getSelection();
      range.selectNodeContents(contentEditableDiv.current);
      if (sel) {
        sel.removeAllRanges();
        sel.addRange(range);
        contentEditableDiv.current.focus();
      }
      setTimeout(() => {
        contentEditableDiv.current!.focus();
      },200)
    }
  },[editMode])

  const objectColor = useMemo(() => {
    return data.isObject ? getRandomStringColor(data.type) : undefined;
  }, [data.isObject, data.type])
  const contextMenuTriggerRef = useRef<HTMLButtonElement>(null);
  return (
    <><ContextMenu>
      <ContextMenuTrigger className='pointer-events-auto hidden' asChild>
        <button ref={contextMenuTriggerRef}></button>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem className='' onClick={(ev) => {
          ev.stopPropagation();
          // contentEditableDiv.current?.focus();

          // setEditMode(true);
          // setTimeout(() => {
          //   contentEditableDiv.current?.focus();
          // },200)
          setEditMode(true);
          // editType(contentEditableDiv.current!);
          // contentEditableDiv.current?.dispatchEvent(new MouseEvent("dblclick", {
          //   bubbles: true,
          //   clientX: ev.clientX,
          //   clientY: ev.clientY,
          // }))
          // flow.deleteElements({nodes: [{id}]});
        }}>Edit Type</ContextMenuItem>
        <ContextMenuItem className='text-red-600 hover:focus:text-red-500' onClick={(ev) => {
          ev.stopPropagation();
          flow.deleteElements({ nodes: [{ id }] });
        }}>Delete Node</ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu><div onContextMenu={(ev) => {
      ev.stopPropagation();
      if (contextMenuTriggerRef.current && !editMode) {
        ev.preventDefault();
        contextMenuTriggerRef.current.dispatchEvent(new MouseEvent("contextmenu", {
          bubbles: true,
          clientX: ev.clientX,
          clientY: ev.clientY,
        }),);
      }
    }} 
    // w-[4rem] and h-[2rem] for small demo images
    className={clsx("border-2 w-[8rem] py-1 px-1 flex items-center justify-center relative min-h-[3.5rem] h-fit bg-white rounded group", !data.isObject && "border-[var(--arrow-primary)]", selected && "shadow-lg")}
      style={{ borderColor: objectColor }}>
        <div className={clsx("border text-center border-transparent flex items-center min-h-[2rem] w-[calc(100%-1rem)]  drag-handle__custom group-hover:border-dashed group-hover:border-gray-300/50 z-2", connection.inProgress && "pointer-events-none")}>

          <div contentEditable={editMode} ref={contentEditableDiv} className='w-full text-xs pointer-events-auto'
            suppressContentEditableWarning={true}
            tabIndex={1}
            onKeyDownCapture={(ev) => {
              if (ev.key === "Enter") {
                ev.preventDefault();
                ev.stopPropagation();
                ev.currentTarget.blur();
              }
            }}
            onMouseDownCapture={(ev) => {
              if (editMode) {
                ev.stopPropagation();
              }
            }}
            onDoubleClick={(ev) => {
              if (editMode) {
                // ev.preventDefault();
                // applyNameEdit(ev);
                ev.stopPropagation();
              } else {
                setEditMode(true);
              }
            }}
            onBlur={(ev) => {
              if(ev.relatedTarget?.role === "menuitem"){
                ev.preventDefault();
                ev.stopPropagation();
                contentEditableDiv.current!.focus();
                return;
              }
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
            type="source" />
        )}
        {/* We want to disable the target handle, if the connection was started from this node */}
        {(!connection.inProgress || isTarget) && (
          <Handle className="customHandle z-10" position={Position.Left} type="target" isConnectableStart={false} />
        )}
      </div></>
  );
}
