// The dialog that Machines and Settings share: a dialog over the page, and a
// bottom sheet on a phone, from the modal frame of Firna. A dialog never
// grows past the screen: its title and its footer stay, and a long body
// scrolls between them, as the mockups show. On a wide window the frame
// scrolls its body by itself. Its sheet scrolls as one piece, so on a phone
// the body scrolls in a view of its own, whose height leaves room for the
// title and the footer.

import { useState, type ReactNode, type RefObject } from "react";
import {
  ScrollView,
  StyleSheet,
  useWindowDimensions,
  View,
} from "react-native";
import { WebModalFrame, type Focusable } from "@firna/ui/modal";

import { useViewport } from "./viewport.ts";

/** The share of the window that the Firna sheet takes at most. */
const sheetShare = 0.92;

/**
 * The parts of the Firna sheet around the body and the footer actions: the
 * grip, a title of up to two lines with its padding, the padding under the
 * body, and the border and padding of the footer.
 */
const sheetChrome = 14 + 80 + 14 + 27 + 2;

/** The least height of a body, for a window too short for the sheet. */
const minimumBody = 120;

/**
 * A dialog over the page, and a bottom sheet on a phone. While its change
 * runs, it cannot close. Point `initialFocusRef` at the safe action of a
 * destructive confirmation.
 */
export function FeatureDialog({
  title,
  visible,
  busy = false,
  onClose,
  footer,
  initialFocusRef,
  children,
}: {
  title: string;
  visible: boolean;
  busy?: boolean;
  onClose: () => void;
  footer: ReactNode;
  initialFocusRef?: RefObject<Focusable | null>;
  children: ReactNode;
}) {
  const sheet = useViewport() === "mobile";
  const { height } = useWindowDimensions();
  const [footerHeight, setFooterHeight] = useState(0);
  const body = Math.max(
    minimumBody,
    Math.floor(height * sheetShare) - sheetChrome - footerHeight,
  );
  return (
    <WebModalFrame
      title={title}
      visible={visible}
      onClose={onClose}
      closeDisabled={busy}
      dismissible={!busy}
      placement={sheet ? "bottom-sheet" : "center"}
      scroll={!sheet}
      size="sm"
      footer={
        <View
          style={styles.footer}
          onLayout={(event) =>
            setFooterHeight(Math.ceil(event.nativeEvent.layout.height))
          }
        >
          {footer}
        </View>
      }
      initialFocusRef={initialFocusRef}
    >
      {sheet ? (
        <ScrollView
          style={{ maxHeight: body }}
          contentContainerStyle={styles.body}
        >
          {children}
        </ScrollView>
      ) : (
        children
      )}
    </WebModalFrame>
  );
}

const styles = StyleSheet.create({
  // The footer of each dialog takes the width of the frame, and its actions
  // keep to the end unless the footer spreads them.
  footer: { flexGrow: 1, alignItems: "flex-end" },
  body: { gap: 12 },
});
