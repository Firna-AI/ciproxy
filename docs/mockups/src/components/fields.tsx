// Form fields of the static mockups, on the Firna input. A read-only field
// shows its value on the soft fill with a lock, such as the name of a label,
// which never changes.

import { Lock } from "lucide-react";
import { Input } from "@firna/ui/input";

import { theme } from "@ciproxy/tokens/firna";
import { noop } from "./actions.tsx";

const readOnlyBox = { backgroundColor: theme.colors.soft };

export function TextField({
  label,
  value,
  placeholder,
  hint,
  error,
  secret,
  readOnly = false,
}: {
  label: string;
  value?: string;
  placeholder?: string;
  hint?: string;
  error?: string;
  secret?: boolean;
  /** The value cannot change, so the field shows it with a lock. */
  readOnly?: boolean;
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
      editable={!readOnly}
      suffixIcon={readOnly ? Lock : undefined}
      style={readOnly ? readOnlyBox : undefined}
    />
  );
}
