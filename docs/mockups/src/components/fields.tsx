// Form fields of the static mockups, on the Firna input.

import { Input } from "@firna/ui/input";

import { noop } from "./actions.tsx";

export function TextField({
  label,
  value,
  placeholder,
  hint,
  error,
  secret,
}: {
  label: string;
  value?: string;
  placeholder?: string;
  hint?: string;
  error?: string;
  secret?: boolean;
}) {
  return (
    <Input
      label={label}
      value={value ?? ""}
      onChangeText={noop}
      placeholder={placeholder}
      hint={hint}
      error={error ?? null}
      invalid={error !== undefined}
      secureTextEntry={secret}
    />
  );
}
