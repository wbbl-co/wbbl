export const graphWorker = new Worker("/web/graph-worker.ts", {
  type: "module",
  credentials: "same-origin",
});
