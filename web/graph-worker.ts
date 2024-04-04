import { WbblGraphWebWorkerJsWrapper } from "../pkg/wbbl";
import type { DeregisterCanvas, RegisterCanvas } from "./worker_message_types";

async function run_in_worker() {
  let web_worker_main = await WbblGraphWebWorkerJsWrapper.new(self, self);
  onmessage = (msg: MessageEvent<any>) => {
    if (!!msg.data.RegisterCanvas) {
      // This message was sent from JS as it cannot be serialized by serde and contains the offscreen
      // canvas handle object.
      const register_msg = msg.data.RegisterCanvas as RegisterCanvas;
      web_worker_main.register_canvas(
        register_msg.nodeId,
        register_msg.offscreenCanvas,
      );
    } else if (!!msg.data.DeregisterCanvas) {
      // This message was sent from JS as while it could be serialized by serde,
      // it felt more symmetrical to treat it consistently to the RegisterCanvas
      // message.
      const deregister_msg = msg.data.DeregisterCanvas as DeregisterCanvas;
      web_worker_main.deregister_canvas(deregister_msg.nodeId);
    } else {
      try {
        web_worker_main.handle_message(msg.data);
      } catch (e) {
        console.error("error", e);
      }
    }
  };
}

run_in_worker();
