// The words that a person reads in place of an action that the role does not
// allow, such as "Ask an Admin". Everyone sees the state, and only the roles
// that may act get the action.

import { Text } from "react-native";

import { text } from "./text.ts";

export function RoleNote({ children }: { children: string }) {
  return <Text style={text.caption}>{children}</Text>;
}
