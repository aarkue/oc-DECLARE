import { AppNode } from "./nodes/types";
export const DATA_TYPE_COLORS = {
    "OCEL": "#ff5dbe",
    "PATH": "#6565ff",
    "STRING": "#5cffae",
    "FLOAT": "#5745ae",
    "OTHER": "#7daaff",
} as const;
export type DataTypes = keyof typeof DATA_TYPE_COLORS;
export type InputOrOutput = {
    id: string,
    type: DataTypes,
    optional?: boolean,
}
export function getInputForStep(stepName: AppNode['data']['stepType']): InputOrOutput[] {
    if (stepName === "import-ocel"){
        return [{id: "path", type: "PATH"},{id: "options", type: "OTHER", optional: true}]
    }
    if (stepName === "filter-ocel") {
        return [{id: "ocel", type: "OCEL"},{id: "fraction", type: "FLOAT"}]
    }
    return []
}


export function getOutputsForStep(stepName: AppNode['data']['stepType']): InputOrOutput[] {
    if (stepName === "import-ocel"){
        return [{id: "ocel", type: "OCEL"}]
    }
    if (stepName === "filter-ocel") {
        return [{id: "ocel", type: "OCEL"}]
    }
    return []
}