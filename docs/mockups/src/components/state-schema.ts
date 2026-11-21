// The prop schema of a screen whose saved states differ by one `state` prop.
// A closed enum gives the catalogue a select control with valid states only,
// and TypeScript derives the render's prop type from it.

export function stateSchema<const S extends readonly [string, ...string[]]>(
  states: S,
) {
  return {
    type: "object",
    additionalProperties: false,
    required: ["state"],
    properties: { state: { enum: states } },
  } as const;
}
