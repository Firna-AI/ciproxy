// The order of a label in the label dialog: the places where its jobs run,
// in order. A grab handle moves a place, by pointer or by keyboard, when the
// order holds more than one, and Remove takes a place out. Add a place
// offers each place that the order does not hold: the own machines, and each
// offered provider, which the workspace connected once. Each row says what
// the place does with a job of the size: how long a job waits for the own
// machines, the size of a provider that runs it, or why a job skips the
// place. The overflow wait shows on the own machines only, and only when a
// place follows them. An empty order holds the jobs. A macOS label holds the
// own machines only, with no handle and no Remove.

import { Plus } from "lucide-react";
import { SortableList } from "@firna/ui/sortable-list";

import { Action, noop, TextLink } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { capitalized, holdText } from "../../components/hold-text.ts";
import { paths } from "../../components/paths.ts";
import { placeName, type Place } from "../../fixtures/label-order.ts";
import { overflow } from "../../fixtures/overflow.ts";
import { holdOf, type PlaceContext } from "../../fixtures/placement.ts";
import { providerName, type ProviderKey } from "../../fixtures/providers.ts";
import { fitsOwnMachine, type Label } from "../../fixtures/settings.ts";
import { providerSizeFor, sizePhrase } from "../../fixtures/sizes.ts";

/** What a place does with a job of the label, under its name. */
function placeCaption(
  place: Place,
  index: number,
  label: Label,
  context: PlaceContext,
): string {
  if (place === "machines") {
    if (!fitsOwnMachine(label, context.machines)) {
      return "No machine of yours fits this size";
    }
    if (label.platform === "macos") return "Jobs wait for a free Mac";
    return index < label.order.length - 1
      ? `Jobs wait up to ${overflow.waitSeconds} seconds for a free slot`
      : "Jobs wait for a free slot";
  }
  const hold = holdOf(place, context);
  const skip = (text: string) => `${capitalized(text)}, so jobs skip it`;
  if (hold === "off" || hold === "not-connected" || hold === "unavailable") {
    return skip(holdText(place, hold));
  }
  const size = providerSizeFor(place, label);
  if (!size) return skip(`no ${providerName(place)} size fits`);
  return hold
    ? skip(holdText(place, hold))
    : `Runs on ${size.name}, ${sizePhrase(size)}`;
}

function OrderRow({
  place,
  index,
  label,
  context,
  locked,
}: {
  place: Place;
  index: number;
  label: Label;
  context: PlaceContext;
  /** The place cannot leave the order, as on a macOS label. */
  locked: boolean;
}) {
  return (
    <span className="order-row">
      <span className="order-index num" aria-hidden="true">
        {index + 1}
      </span>
      <span className="order-text">
        <span className="strong">{placeName(place, label.platform)}</span>
        <span className="caption">
          {placeCaption(place, index, label, context)}
        </span>
      </span>
      {locked ? null : (
        <Action size="sm" tone="ghost">
          Remove
        </Action>
      )}
    </span>
  );
}

/** The places that Add a place lists: the own machines and each offered provider. */
function addable(label: Label, offered: readonly ProviderKey[]): Place[] {
  const places: Place[] = ["machines", ...offered];
  return places.filter((place) => !label.order.includes(place));
}

function AddPlace({
  label,
  offered,
}: {
  label: Label;
  offered: readonly ProviderKey[];
}) {
  const places = addable(label, offered);
  return (
    <>
      {places.length > 0 ? (
        <div className="row-wrap">
          <span className="caption">Add a place</span>
          {places.map((place) => (
            <Action key={place} size="sm" icon={Plus}>
              {placeName(place, label.platform)}
            </Action>
          ))}
        </div>
      ) : null}
      {offered.length === 0 ? (
        <p className="caption">
          To add a provider, connect it in{" "}
          <TextLink to={paths.overflowNotConnected} newTab>
            Settings under Overflow
          </TextLink>
          .
        </p>
      ) : null}
    </>
  );
}

export function LabelOrder({
  label,
  context,
  offered,
}: {
  label: Label;
  context: PlaceContext;
  /** The providers that the workspace connected once, which an order can hold. */
  offered: readonly ProviderKey[];
}) {
  const macos = label.platform === "macos";
  /** A place can move when the order holds another one. */
  const movable = !macos && label.order.length > 1;
  return (
    <div className="stack-tight">
      <p className="small strong">Where jobs run</p>
      <p className="caption">
        {macos
          ? "A macOS label runs on your Macs only."
          : "A job goes to the next place when a place cannot take it."}
      </p>
      {label.order.length === 0 ? (
        <Notice tone="warning" title="Jobs with this label wait">
          The order has no place. Add a place to run them.
        </Notice>
      ) : (
        <SortableList
          accessibilityLabel={`Where jobs of ${label.name} run`}
          items={[...label.order]}
          itemKey={(place) => place}
          itemLabel={(place) => placeName(place, label.platform)}
          handle={movable ? "start" : undefined}
          onReorder={movable ? noop : undefined}
          size="sm"
          renderItem={(place, index) => (
            <OrderRow
              place={place}
              index={index}
              label={label}
              context={context}
              locked={macos}
            />
          )}
        />
      )}
      {macos ? null : <AddPlace label={label} offered={offered} />}
    </div>
  );
}
