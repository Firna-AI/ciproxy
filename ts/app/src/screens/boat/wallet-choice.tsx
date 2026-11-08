// The wallet choice of a Boat connection, after the key check: the personal
// wallet and each Boat organization wallet that the key can use, with the
// likely wallet preselected. Setup and Settings under Overflow use this one
// component, for a first connection, a reconnect, and Change key. It shows
// only what its caller passes: the caller takes the choices and the likely
// wallet from the key check of the API, and drops them when the key changes.

import { StyleSheet, Text, View } from "react-native";
import {
  RadioCard,
  RadioCardGroup,
  type RadioCardProps,
} from "@firna/ui/radio";

import { text } from "../../components/text.ts";

/** One wallet as a radio card, keyed by the value that the caller saves. */
export type WalletOption = Pick<RadioCardProps, "title" | "body"> & {
  value: string;
};

/** The line under the name of the personal wallet. */
export const personalWalletText = "Personal wallet";

/** The line under the name of a Boat organization wallet. */
export const organizationWalletText = "Boat organization wallet";

export function WalletChoice({
  options,
  value,
  disabled,
  onChange,
}: {
  options: readonly WalletOption[];
  /** The picked wallet: at first the likely wallet of the key check. */
  value: string | null;
  /** A Boat change of the workspace runs, so the choice waits. */
  disabled: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <View style={styles.choice}>
      <RadioCardGroup label="Wallet that Boat bills">
        {options.map((option) => (
          <RadioCard
            key={option.value}
            title={option.title}
            body={option.body}
            checked={option.value === value}
            disabled={disabled}
            onPress={() => onChange(option.value)}
          />
        ))}
      </RadioCardGroup>
      <Text style={text.caption}>
        Your key can use these wallets. Each Boat organization has its own
        wallet.
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  choice: { gap: 6 },
});
