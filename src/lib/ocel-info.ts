import { createContext } from "react";

export type OCELInfo = Record<string,Record<string,{min: number, max: number}>>;

export const OCELInfoContext = createContext<{ocelInfo: OCELInfo, setOcelInfo: (ocelInfo: OCELInfo) => unknown}>({ocelInfo: {}, setOcelInfo: () => {}});