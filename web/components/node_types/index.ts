import BinaryOperatorNode from "./BinaryOperatorNode";
import BuiltInNode from "./BuiltInNode";
import OutputNode from "./OutputNode";
import PreviewNode from "./PreviewNode";
import SlabNode from "./SlabNode";

export const nodeTypes = {
  output: OutputNode,
  slab: SlabNode,
  preview: PreviewNode,
  add: BinaryOperatorNode,
  subtract: BinaryOperatorNode,
  multiply: BinaryOperatorNode,
  divide: BinaryOperatorNode,
  ">": BinaryOperatorNode,
  ">=": BinaryOperatorNode,
  "<": BinaryOperatorNode,
  "<=": BinaryOperatorNode,
  "==": BinaryOperatorNode,
  "!=": BinaryOperatorNode,
  modulo: BinaryOperatorNode,
  and: BinaryOperatorNode,
  or: BinaryOperatorNode,
  "<<": BinaryOperatorNode,
  ">>": BinaryOperatorNode,
  position: BuiltInNode,
  normal: BuiltInNode,
  tangent: BuiltInNode,
  bitangent: BuiltInNode,
  clip_pos: BuiltInNode,
  tex_coord: BuiltInNode,
  tex_coord_2: BuiltInNode,
};
