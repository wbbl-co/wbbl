import { Position, getBezierPath, getSmoothStepPath } from "@xyflow/system";
import { EdgeStyle } from "../../pkg/wbbl";
import { EDGE_STROKE_WIDTH, VECTOR_EDGE_STROKE_WIDTH } from "../port-constants";

export function defaultConnectionPathProvider(
  startPos: { x: number; y: number },
  endPos: { x: number; y: number },
  edgeStyle: EdgeStyle,
) {
  return (position: Float32Array) => {
    if (edgeStyle === EdgeStyle.Default) {
      return `M ${startPos.x + position[0]} ${startPos.y + position[1]} L ${endPos.x + position[0]} ${endPos.y + position[1]}`;
    } else if (edgeStyle === EdgeStyle.Bezier) {
      return getBezierPath({
        sourceX: startPos.x,
        sourceY: startPos.y,
        targetX: endPos.x,
        targetY: endPos.y,
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
      })[0];
    } else {
      return getSmoothStepPath({
        sourceX: startPos.x,
        sourceY: startPos.y,
        targetX: endPos.x,
        targetY: endPos.y,
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
      })[0];
    }
  };
}

export function setConnectionPath(
  ropePath: SVGPathElement,
  edgeClassName: string,
  get_path: (position: Float32Array, zoom: number) => string,
  factorX: number,
  factorY: number,
) {
  if (!!edgeClassName && edgeClassName.includes("S2")) {
    ropePath.style.strokeWidth = VECTOR_EDGE_STROKE_WIDTH + "px";
    ropePath.setAttribute(
      "d",
      `${get_path(
        new Float32Array([
          -factorX * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
          -factorY * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        1,
      )} ${get_path(
        new Float32Array([
          factorX * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
          factorY * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        1,
      )}`,
    );
  } else if (!!edgeClassName && edgeClassName.includes("S3")) {
    ropePath.style.strokeWidth = VECTOR_EDGE_STROKE_WIDTH + "px";
    ropePath.setAttribute(
      "d",
      `${get_path(
        new Float32Array([
          -factorX * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
          -factorY * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        1,
      )} ${get_path(new Float32Array([0, 0]), 1)} ${get_path(
        new Float32Array([
          factorX * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
          factorY * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        1,
      )}`,
    );
  } else if (!!edgeClassName && edgeClassName.includes("S4")) {
    ropePath.style.strokeWidth = VECTOR_EDGE_STROKE_WIDTH + "px";
    ropePath.setAttribute(
      "d",
      `${get_path(
        new Float32Array([
          -factorX * 4 * VECTOR_EDGE_STROKE_WIDTH,
          -factorY * 4 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        1,
      )} ${get_path(
        new Float32Array([
          -factorX * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
          -factorY * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        1,
      )} ${get_path(
        new Float32Array([
          factorX * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
          factorY * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        1,
      )} ${get_path(
        new Float32Array([
          factorX * 4 * VECTOR_EDGE_STROKE_WIDTH,
          factorY * 4 * VECTOR_EDGE_STROKE_WIDTH,
        ]),
        1,
      )}`,
    );
  } else {
    ropePath.style.strokeWidth = EDGE_STROKE_WIDTH + "px";
    ropePath.setAttribute("d", get_path(new Float32Array([0, 0]), 1));
  }
}
