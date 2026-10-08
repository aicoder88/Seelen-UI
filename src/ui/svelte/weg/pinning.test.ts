import { strict as assert } from "node:assert";
import { test } from "node:test";
import { hasAnyPin, isPinnedOnMonitor, withMonitorPin } from "./pinning.ts";

test("legacy shared pins can be hidden on one monitor without changing another", () => {
  const legacy = { pinned: true };
  const edited = withMonitorPin(legacy, "left", false);
  assert.equal(isPinnedOnMonitor(edited, "left"), false);
  assert.equal(isPinnedOnMonitor(edited, "right"), true);
  assert.equal(isPinnedOnMonitor(legacy, "left"), true);
});

test("local pins survive app closure and serialization without leaking to other screens", () => {
  const edited = withMonitorPin({ pinned: false }, "physical-display-A", true);
  const restored = JSON.parse(JSON.stringify(edited));
  assert.equal(hasAnyPin(restored), true);
  assert.equal(isPinnedOnMonitor(restored, "physical-display-A"), true);
  assert.equal(isPinnedOnMonitor(restored, "physical-display-B"), false);
  assert.equal(hasAnyPin(withMonitorPin(restored, "physical-display-A", false)), false);
});

test("changing a pin preserves other monitor overrides", () => {
  let item = withMonitorPin({ pinned: false }, "left", true);
  item = withMonitorPin(item, "right", true);
  item = withMonitorPin(item, "left", false);
  assert.equal(isPinnedOnMonitor(item, "right"), true);
  assert.equal(hasAnyPin(item), true);
});
