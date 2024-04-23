export const graphWorker = new Worker(
  new URL("/web/graph-worker.js", import.meta.url),
  {
    type: "module",
    credentials: "same-origin",
  },
);
