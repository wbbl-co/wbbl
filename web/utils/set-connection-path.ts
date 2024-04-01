import { Viewport } from "@xyflow/system";
import { WbblRope } from "../../pkg/wbbl";
import { EDGE_STROKE_WIDTH, VECTOR_EDGE_STROKE_WIDTH } from "../port-constants";

export function setConnectionPath(
  ropePath: SVGPathElement,
  viewport: Viewport,
  edgeClassName: string,
  rope: WbblRope,
  factorX: number,
  factorY: number,
) {
  if (!!edgeClassName && edgeClassName.includes("S2")) {
    ropePath.style.strokeWidth =
      viewport.zoom * VECTOR_EDGE_STROKE_WIDTH + "px";
    ropePath.setAttribute(
      "d",
      `${rope.get_path(
        new Float32Array([
          viewport.x - factorX * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
          viewport.y - factorY * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        viewport.zoom,
      )} ${rope.get_path(
        new Float32Array([
          viewport.x + factorX * 2 * viewport.zoom * VECTOR_EDGE_STROKE_WIDTH,
          viewport.y + factorY * 2 * viewport.zoom * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        viewport.zoom,
      )}`,
    );
  } else if (!!edgeClassName && edgeClassName.includes("S3")) {
    ropePath.style.strokeWidth =
      viewport.zoom * VECTOR_EDGE_STROKE_WIDTH + "px";
    ropePath.setAttribute(
      "d",
      `${rope.get_path(
        new Float32Array([
          viewport.x - factorX * viewport.zoom * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
          viewport.y - factorY * viewport.zoom * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        viewport.zoom,
      )} ${rope.get_path(
        new Float32Array([viewport.x, viewport.y]),
        viewport.zoom,
      )} ${rope.get_path(
        new Float32Array([
          viewport.x + factorX * viewport.zoom * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
          viewport.y + factorY * viewport.zoom * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        viewport.zoom,
      )}`,
    );
  } else if (!!edgeClassName && edgeClassName.includes("S4")) {
    ropePath.style.strokeWidth =
      viewport.zoom * VECTOR_EDGE_STROKE_WIDTH + "px";
    ropePath.setAttribute(
      "d",
      `${rope.get_path(
        new Float32Array([
          viewport.x - factorX * viewport.zoom * 4 * VECTOR_EDGE_STROKE_WIDTH,
          viewport.y - factorY * viewport.zoom * 4 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        viewport.zoom,
      )} ${rope.get_path(
        new Float32Array([
          viewport.x - factorX * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
          viewport.y - factorY * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        viewport.zoom,
      )} ${rope.get_path(
        new Float32Array([
          viewport.x + factorX * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
          viewport.y + factorY * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        viewport.zoom,
      )} ${rope.get_path(
        new Float32Array([
          viewport.x + factorX * viewport.zoom * 4 * VECTOR_EDGE_STROKE_WIDTH,
          viewport.y + factorY * viewport.zoom * 4 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        viewport.zoom,
      )}`,
    );
  } else {
    ropePath.style.strokeWidth = viewport.zoom * EDGE_STROKE_WIDTH + "px";
    ropePath.setAttribute(
      "d",
      rope.get_path(new Float32Array([viewport.x, viewport.y]), viewport.zoom),
    );
  }
}
