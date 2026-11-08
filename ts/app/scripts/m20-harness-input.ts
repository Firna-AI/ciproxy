import type { components } from "../../api/index.ts";
import { fields } from "./harness-state.ts";

type Schemas = components["schemas"];

export function labelInput(value: unknown): Schemas["LabelInput"] | undefined {
  if (!fields(value)) return undefined;
  const {
    name,
    vcpus,
    memory_gb,
    disk_gb,
    allow_machines,
    allow_boat,
    when_nothing_fits,
  } = value;
  if (
    typeof name !== "string" ||
    typeof vcpus !== "number" ||
    typeof memory_gb !== "number" ||
    typeof disk_gb !== "number" ||
    ![vcpus, memory_gb, disk_gb].every(
      (number) => Number.isInteger(number) && number >= 1,
    ) ||
    typeof allow_machines !== "boolean" ||
    typeof allow_boat !== "boolean" ||
    (when_nothing_fits !== "wait" && when_nothing_fits !== "largest_boat")
  )
    return undefined;
  return {
    name,
    vcpus,
    memory_gb,
    disk_gb,
    allow_machines,
    allow_boat,
    when_nothing_fits,
  };
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
