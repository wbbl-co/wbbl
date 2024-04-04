import { createContext } from "react";

export class PortRefStore {
  elements: Map<string, HTMLElement> = new Map();

  public add(portId: string, ref: HTMLElement) {
    this.elements.set(portId, ref);
  }

  public remove(portId: string) {
    this.elements.delete(portId);
  }

  public get(portId: string) {
    return this.elements.get(portId);
  }
}

export const PortRefStoreContext = createContext<PortRefStore>(
  new PortRefStore(),
);
