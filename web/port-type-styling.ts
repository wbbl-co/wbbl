export function getStyleForPortType(portType: unknown): string {
  if (portType === undefined) {
    return "";
  }
  if (typeof portType === "string") {
    return portType;
  } else {
    let result = ``;
    for (const [key, value] of Object.entries(
      portType as { [s: string]: unknown },
    )) {
      result += `${key} ${getStyleForPortType(value)}`;
    }
    return result;
  }
}
