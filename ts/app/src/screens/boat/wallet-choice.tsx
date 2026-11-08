// The wallet choice of a Boat connection, after the key check: the personal
// wallet and each Boat organization wallet that the key can use, with the
// likely wallet of the API answer picked. Setup and Settings under Overflow
// use this one component, for a first connection, a reconnect, and Change
// key. It shows only the answer of the key check that its form holds.

import { StyleSheet, Text, View } from "react-native";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";

import { colors } from "@ciproxy/tokens";

import { text } from "../../components/text.ts";
import type { WalletOption } from "../../view/boat.ts";

export function WalletChoice({
  options,
  value,
  error,
  disabled,
  onChange,
}: {
  options: readonly WalletOption[];
  /** The chosen wallet identity, or null when none is chosen. */
  value: string | null;
  /** Connect Boat found no chosen wallet. */
  error: string | null;
  /** A Boat change of the workspace runs, so the choice waits. */
  disabled: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <View style={styles.choice}>
      <RadioCardGroup label="Wallet that Boat bills" invalid={error !== null}>
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
      {error === null ? null : (
        <Text style={[text.small, styles.error]} role="alert">
          {error}
        </Text>
      )}
      <Text style={text.caption}>
        Your key can use these wallets. Each Boat organization has its own
        wallet.
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  choice: { gap: 6 },
  error: { color: colors.roseDeep },
});
