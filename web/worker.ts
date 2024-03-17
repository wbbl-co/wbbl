import { WbblGraphWebWorkerMain } from "../pkg/wbbl";
import { WbblGraphWebWorkerMessageType } from "./worker_message_types";
console.log("Hello from worker");

async function run_in_worker() {
  // Loading wasm file
  let web_worker_main = WbblGraphWebWorkerMain.new();
  onmessage = (msg: MessageEvent<WbblGraphWebWorkerMessageType>) => {
    console.log(msg);
    web_worker_main
      .render(msg.data.offscreenCanvas)
      .then(() => console.log("done"))
      .catch((err) => console.log(err));
  };
}

run_in_worker();
