/** Legacy pins remain shared until a monitor explicitly overrides them. */
export interface MonitorPin {
  pinned: boolean;
  pinOverrides?: Record<string, boolean>;
}

export function isPinnedOnMonitor(item: MonitorPin, monitorId: string): boolean {
  return item.pinOverrides?.[monitorId] ?? item.pinned;
}

export function hasAnyPin(item: MonitorPin): boolean {
  return item.pinned || Object.values(item.pinOverrides ?? {}).some(Boolean);
}

export function withMonitorPin<T extends MonitorPin>(item: T, monitorId: string, pinned: boolean): T {
  return { ...item, pinOverrides: { ...item.pinOverrides, [monitorId]: pinned } };
}
