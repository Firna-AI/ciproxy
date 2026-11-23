import type { components } from "../../api/index.ts";
import { readLabelPlace } from "../src/api/parse-label-facts.ts";
import { ParseError } from "../src/api/parse-core.ts";
import { boatPlace, machinePlace } from "./m20-fixtures-labels.ts";
import { fields } from "./harness-state.ts";

type Schemas = components["schemas"];

export function labelInput(
  value: unknown,
  create: boolean,
): Schemas["LabelInput"] | undefined {
  if (!fields(value)) return undefined;
  const {
    name,
    vcpus,
    memory_gb,
    disk_gb,
    order,
    when_nothing_fits,
    platform,
  } = value;
  if (
    platform !== undefined &&
    platform !== "linux_x64" &&
    platform !== "macos_arm64"
  )
    return undefined;
  if (
    typeof name !== "string" ||
    typeof vcpus !== "number" ||
    typeof memory_gb !== "number" ||
    typeof disk_gb !== "number" ||
    ![vcpus, memory_gb, disk_gb].every(
      (number) => Number.isInteger(number) && number >= 1,
    ) ||
    (when_nothing_fits !== "wait" && when_nothing_fits !== "largest_overflow")
  )
    return undefined;
  if (!Array.isArray(order) && !(create && order == null)) return undefined;
  try {
    return {
      ...(platform === undefined ? {} : { platform }),
      name,
      vcpus,
      memory_gb,
      disk_gb,
      order: Array.isArray(order)
        ? order.map(readLabelPlace)
        : platform === "macos_arm64"
          ? [machinePlace]
          : [machinePlace, boatPlace],
      when_nothing_fits,
    };
  } catch (error) {
    if (error instanceof ParseError) return undefined;
    throw error;
  }
}

export function repositoryInput(
  value: unknown,
): Schemas["RepositorySelection"] | undefined {
  if (!fields(value)) return undefined;
  if (value.mode === "all_private") return { mode: "all_private" };
  if (
    value.mode === "selected" &&
    Array.isArray(value.ids) &&
    value.ids.every((id: unknown) => typeof id === "string")
  )
    return { mode: "selected", ids: value.ids };
  return undefined;
}

export function roleInput(value: unknown): Schemas["RoleChange"] | undefined {
  if (!fields(value)) return undefined;
  if (
    value.role === "admin" ||
    value.role === "billing" ||
    value.role === "viewer"
  )
    return { role: value.role };
  return undefined;
}
