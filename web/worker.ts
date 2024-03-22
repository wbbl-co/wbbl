// import { WbblGraphWebWorkerMain } from "../pkg/wbbl";
// import type { WbblGraphWebWorkerMessageType } from "./worker_message_types";

// async function run_in_worker() {
//   // Loading wasm file
//   let web_worker_main = WbblGraphWebWorkerMain.new();
//   onmessage = (msg: MessageEvent<WbblGraphWebWorkerMessageType>) => {
//     console.log(msg);
//     web_worker_main
//       .render(msg.data.offscreenCanvas)
//       .then(() => console.log("done"))
//       .catch((err) => console.log(err));
//   };
// }

// run_in_worker();
