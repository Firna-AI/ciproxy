// The labels of the what-ifs of the label order. With two providers, Second
// provider joined the end of the order of each Linux label when mokly-ai
// connected it on 9 November, and an Admin moved it before Boat in the order
// of ciproxy-large: its cheapest size that fits the label has 8 vCPU, where
// the Boat size has 16. In other what-ifs with Boat only, an Admin removes
// both places of ciproxy-medium, puts Boat before the own machines for
// ciproxy-large, so that its long jobs leave the own slots to short ones, or
// removes the own machines from ciproxy-16vcpu, which no machine fits.

import type { Place } from "./label-order.ts";
import { boatOnlyLabel, labelNamed, labels, type Label } from "./settings.ts";

/** The order of a Linux label after Second provider joined its end. */
const machinesBoatSecond: readonly Place[] = ["machines", "boat", "second"];

/** ciproxy-large with two providers: Second provider before Boat. */
export const largeSecondFirst: Label = {
  ...labelNamed("ciproxy-large"),
  order: ["machines", "second", "boat"],
};

/** ciproxy with two providers: the own machines, Boat, then Second provider. */
export const defaultSecondLast: Label = {
  ...labelNamed("ciproxy"),
  order: machinesBoatSecond,
};

/** The labels of mokly-ai in the what-ifs with two providers. */
export const twoProviderLabels: readonly Label[] = labels.map((label) =>
  label.name === largeSecondFirst.name
    ? largeSecondFirst
    : { ...label, order: machinesBoatSecond },
);

/** ciproxy-medium after an Admin removed both of its places. */
export const emptyOrderLabel: Label = {
  ...labelNamed("ciproxy-medium"),
  order: [],
};

/** ciproxy-large with Boat before the own machines. */
export const boatFirstLabel: Label = {
  ...labelNamed("ciproxy-large"),
  order: ["boat", "machines"],
};

/** ciproxy-16vcpu, which no own machine fits, with Boat as its one place. */
export const providersOnlyLabel: Label = {
  ...boatOnlyLabel,
  order: ["boat"],
};
