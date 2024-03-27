import { Fragment, useCallback, useState } from "react";
import { Combobox, Dialog, Transition } from "@headlessui/react";
import { MagnifyingGlassIcon } from "@heroicons/react/20/solid";
import {
  DocumentIcon,
  ExclamationCircleIcon,
} from "@heroicons/react/24/outline";
import { WbblWebappNodeType } from "../../pkg/wbbl";

const items = [
  {
    node_type: WbblWebappNodeType.Slab,
    name: "Slab",
    description: "Primary PBR Shader Node. Can be mixed with other slabs",
    url: "#",
    color: "bg-blue",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Preview,
    name: "Preview",
    description: "Visualises Input Values",
    url: "#",
    color: "bg-black",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Add,
    name: "Add",
    description: "Adds Values Together",
    url: "#",
    color: "bg-orange",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Subtract,
    name: "Subtract",
    description: "Subtracts Values From One Another",
    url: "#",
    color: "bg-orange",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Multiply,
    name: "Multiply",
    description: "Multiplies Values Together",
    url: "#",
    color: "bg-orange",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Divide,
    name: "Divide",
    description: "Divides Values By One Another",
    url: "#",
    color: "bg-orange",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Modulo,
    name: "Modulo",
    description: "Returns the remainder of two values",
    url: "#",
    color: "bg-orange",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Greater,
    name: ">",
    description: "Returns whether x is greater than y",
    url: "#",
    color: "bg-orange",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.GreaterEqual,
    name: ">=",
    description: "Returns whether x is greater or equal to y",
    url: "#",
    color: "bg-lime",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Less,
    name: "<",
    description: "Returns whether x is less than y",
    url: "#",
    color: "bg-lime",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.LessEqual,
    name: "<=",
    description: "Returns whether x is lesser or equal to y",
    url: "#",
    color: "bg-lime",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Equal,
    name: "==",
    description: "Returns whether x is equal to y",
    url: "#",
    color: "bg-lime",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.NotEqual,
    name: "!=",
    description: "Returns whether x is not equal to y",
    url: "#",
    color: "bg-lime",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.And,
    name: "And",
    description:
      "If x and y are booleans, returns whether they are both true, else if a number, computes the logical conjunction of their bits",
    url: "#",
    color: "bg-dustPink",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.Or,
    name: "Or",
    description:
      "If x and y are booleans, returns whether either are true, else if a number, computes the logical intersection of their bits",
    url: "#",
    color: "bg-dustPink",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.ShiftLeft,
    name: "<<",
    description: "Returns the bits of x bit shifted left y times",
    url: "#",
    color: "bg-dustPink",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.ShiftRight,
    name: ">>",
    description: "Returns the bits of x bit shifted right y times",
    url: "#",
    color: "bg-dustPink",
    icon: DocumentIcon,
  },

  {
    node_type: WbblWebappNodeType.WorldPosition,
    name: "World Position",
    description: "Returns the position of the texel in world space",
    url: "#",
    color: "bg-green",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.WorldNormal,
    name: "World Normal",
    description: "Returns the normal of the texel in world space",
    url: "#",
    color: "bg-green",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.WorldTangent,
    name: "World Tangent",
    description: "Returns the tangent of the texel in world space",
    url: "#",
    color: "bg-green",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.WorldBitangent,
    name: "World Bitangent",
    description: "Returns the bitangent of the texel in world space",
    url: "#",
    color: "bg-green",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.ClipPosition,
    name: "Clip Position",
    description: "Returns the position of the texel in clip space",
    url: "#",
    color: "bg-green",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.TexCoord,
    name: "Tex Coord",
    description: "Returns the 1st texture coordinate of this model",
    url: "#",
    color: "bg-green",
    icon: DocumentIcon,
  },
  {
    node_type: WbblWebappNodeType.TexCoord2,
    name: "Tex Coord 2",
    description: "Returns the 1st texture coordinate of this model, if present",
    url: "#",
    color: "bg-green",
    icon: DocumentIcon,
  },
] as const;

function classNames(...classes: (boolean | string)[]) {
  return classes.filter(Boolean).join(" ");
}

export const NODE_MENU_DIMENSIONS = { width: 600, height: 400 } as const;
export default function NodeMenu(props: {
  open: boolean;
  onClose: (open: boolean) => void;
  position: null | {
    x: number;
    y: number;
    top?: number;
    left?: number;
    bottom?: number;
    right?: number;
  };
  addNode: (type: WbblWebappNodeType, x: number, y: number) => void;
}) {
  const [query, setQuery] = useState("");

  const filteredItems =
    query === ""
      ? null
      : items.filter((item) => {
          return (
            item.name.toLowerCase().includes(query.toLowerCase()) ||
            item.description.toLowerCase().includes(query.toLowerCase())
          );
        });

  const onSelect = useCallback(
    (value: { node_type: WbblWebappNodeType }) => {
      props.addNode(value.node_type, props.position!.x, props.position!.y);
      props.onClose(false);
    },
    [props.addNode, props.position, props.onClose],
  );
  return (
    <Transition.Root
      show={props.open}
      as={Fragment}
      afterLeave={() => setQuery("")}
      appear
    >
      <Dialog as="div" className="relative z-10" onClose={props.onClose}>
        <div className="fixed inset-0 z-10 h-screen w-screen overflow-hidden">
          <Transition.Child
            as={Fragment}
            enter="ease-out duration-300"
            enterFrom="opacity-0 scale-95"
            enterTo="opacity-100 scale-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100 scale-100"
            leaveTo="opacity-0 scale-95"
          >
            <Dialog.Panel
              style={
                props.position == null
                  ? {
                      width: NODE_MENU_DIMENSIONS.width - 30,
                      maxHeight: NODE_MENU_DIMENSIONS.height - 30,
                    }
                  : {
                      ...props.position,
                      width: NODE_MENU_DIMENSIONS.width - 30,
                      maxHeight: NODE_MENU_DIMENSIONS.height - 30,
                    }
              }
              id="node-menu"
              className="absolute transform divide-y divide-gray-100 overflow-hidden rounded-xl bg-white shadow-2xl ring-1 ring-black ring-opacity-5 transition-all"
            >
              <Combobox<(typeof items)[0]> onChange={onSelect}>
                <div className="relative bg-white">
                  <MagnifyingGlassIcon
                    className="pointer-events-none absolute left-4 top-3.5 h-5 w-5 text-gray-400"
                    aria-hidden="true"
                  />
                  <Combobox.Input
                    className="h-12 w-full border-0 bg-transparent pl-11 pr-4 text-gray-900 placeholder:text-gray-400 focus:ring-0 sm:text-sm"
                    placeholder="Search..."
                    onChange={(event) => setQuery(event.target.value)}
                  />
                </div>

                {(filteredItems == null || filteredItems.length > 0) && (
                  <Combobox.Options
                    static
                    className="max-h-96 transform-gpu overflow-y-auto p-3 pb-20"
                  >
                    {(filteredItems == null ? items : filteredItems).map(
                      (item) => (
                        <Combobox.Option
                          key={item.node_type}
                          value={item}
                          className={({ active }) =>
                            classNames(
                              "flex cursor-default select-none rounded-xl p-3",
                              active && "bg-gray-100",
                            )
                          }
                        >
                          {({ active }) => (
                            <>
                              <div
                                className={classNames(
                                  "flex h-10 w-10 flex-none items-center justify-center rounded-lg",
                                  item.color,
                                )}
                              >
                                <item.icon
                                  className="h-6 w-6 text-white"
                                  aria-hidden="true"
                                />
                              </div>
                              <div className="ml-4 flex-auto">
                                <p
                                  className={classNames(
                                    "text-sm font-medium",
                                    active ? "text-gray-900" : "text-gray-700",
                                  )}
                                >
                                  {item.name}
                                </p>
                                <p
                                  className={classNames(
                                    "text-sm",
                                    active ? "text-gray-700" : "text-gray-500",
                                  )}
                                >
                                  {item.description}
                                </p>
                              </div>
                            </>
                          )}
                        </Combobox.Option>
                      ),
                    )}
                  </Combobox.Options>
                )}

                {query !== "" &&
                  filteredItems !== null &&
                  filteredItems.length === 0 && (
                    <div className="px-6 py-14 text-center text-sm sm:px-14">
                      <ExclamationCircleIcon
                        type="outline"
                        name="exclamation-circle"
                        className="mx-auto h-6 w-6 text-gray-400"
                      />
                      <p className="mt-4 font-semibold text-gray-900">
                        No results found
                      </p>
                      <p className="mt-2 text-gray-500">
                        No nodes found for this search term. Please try again.
                      </p>
                    </div>
                  )}
              </Combobox>
            </Dialog.Panel>
          </Transition.Child>
        </div>
      </Dialog>
    </Transition.Root>
  );
}
