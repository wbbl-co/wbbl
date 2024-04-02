import BinaryOperatorNode from "./BinaryOperatorNode";
import BuiltInNode from "./BuiltInNode";
import OutputNode from "./OutputNode";
import PreviewNode from "./PreviewNode";
import SlabNode from "./SlabNode";
import { WbblWebappNodeType } from "../../../pkg/wbbl";

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

export type NodeCategory =
  | "output"
  | "utility"
  | "math"
  | "material-category"
  | "logic"
  | "builtins";

export const nodeMetaData: {
  [K in keyof typeof nodeTypes]: {
    category: NodeCategory;
    type: WbblWebappNodeType;
    nodeMenuName?: string;
    description: string;
    hiddenFromNodeMenu?: boolean;
  };
} = {
  slab: {
    category: "material-category",
    type: WbblWebappNodeType.Slab,
    description: "Primary PBR Shader Node. Can be mixed with other slabs",
  },
  output: {
    category: "output",
    type: WbblWebappNodeType.Output,
    description: "The material output",
    hiddenFromNodeMenu: true,
  },
  preview: {
    category: "utility",
    type: WbblWebappNodeType.Preview,
    description: "Visualises Input Values",
  },
  add: {
    category: "math",
    type: WbblWebappNodeType.Add,
    description: "Adds values together",
  },
  subtract: {
    category: "math",
    type: WbblWebappNodeType.Subtract,
    description: "Subtracts values from One Another",
  },
  multiply: {
    category: "math",
    type: WbblWebappNodeType.Multiply,
    description: "Multiplies values together",
  },
  divide: {
    category: "math",
    type: WbblWebappNodeType.Divide,
    description: "Divides Values By One Another",
  },
  ">": {
    category: "logic",
    type: WbblWebappNodeType.Greater,
    description: "Returns whether x is greater than y",
  },
  ">=": {
    category: "logic",
    type: WbblWebappNodeType.GreaterEqual,
    description: "Returns whether x is greater than or equal to y",
  },
  "<": {
    category: "logic",
    type: WbblWebappNodeType.Less,
    description: "Returns whether x is less than y",
  },
  "<=": {
    category: "logic",
    type: WbblWebappNodeType.LessEqual,
    description: "Returns whether x is less than or equal to y",
  },
  "==": {
    category: "logic",
    type: WbblWebappNodeType.Equal,
    description: "Returns whether x is equal to y",
  },
  "!=": {
    category: "logic",
    type: WbblWebappNodeType.NotEqual,
    description: "Returns whether x is not equal to y",
  },
  modulo: {
    category: "math",
    type: WbblWebappNodeType.Modulo,
    description: "Returns the remainder of two values",
  },
  and: {
    category: "logic",
    type: WbblWebappNodeType.And,
    description:
      "If x and y are booleans, returns whether they are both true, else if a number, computes the logical conjunction of their bits",
  },
  or: {
    category: "logic",
    type: WbblWebappNodeType.Or,
    description:
      "If x and y are booleans, returns whether either are true, else if a number, computes the logical intersection of their bits",
  },
  "<<": {
    category: "math",
    type: WbblWebappNodeType.ShiftLeft,
    description: "Returns the bits of x bit shifted left y times",
  },
  ">>": {
    category: "math",
    type: WbblWebappNodeType.ShiftRight,
    description: "Returns the bits of x bit shifted right y times",
  },
  position: {
    nodeMenuName: "World Position",
    category: "builtins",
    type: WbblWebappNodeType.WorldPosition,
    description: "Returns the position of the texel in world space",
  },
  normal: {
    nodeMenuName: "World Normal",
    category: "builtins",
    type: WbblWebappNodeType.WorldNormal,
    description: "Returns the normal of the texel in world space",
  },
  tangent: {
    nodeMenuName: "World Tangent",
    category: "builtins",
    type: WbblWebappNodeType.WorldTangent,
    description: "Returns the tangent of the texel in world space",
  },
  bitangent: {
    nodeMenuName: "World Bitangent",
    category: "builtins",
    type: WbblWebappNodeType.WorldBitangent,
    description: "Returns the bitangent of the texel in world space",
  },
  clip_pos: {
    nodeMenuName: "Clip Position",
    category: "builtins",
    type: WbblWebappNodeType.ClipPosition,
    description: "Returns the position of the texel in clip space",
  },
  tex_coord: {
    nodeMenuName: "Texture Coordinate",
    category: "builtins",
    type: WbblWebappNodeType.TexCoord,
    description: "Returns the 1st texture coordinate for this model",
  },
  tex_coord_2: {
    nodeMenuName: "Texture Coordinate 2",
    category: "builtins",
    type: WbblWebappNodeType.TexCoord2,
    description:
      "Returns the 2nd texture coordinate for this model, if present",
  },
};
