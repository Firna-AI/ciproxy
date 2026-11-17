// The cache hits of the last 7 days among the figures of Machines: the share
// of the machines that measured their machine cache, with a line that names
// each machine that the share leaves out, or Unknown when no machine measured
// it. Unknown has no line, because the share then leaves out every machine.

import { Fragment } from "react";
import { StyleSheet, Text, View } from "react-native";

import { text } from "../../components/text.ts";
import type { CacheHits } from "../../view/machines-cache.ts";
import { cacheCopy, listCopy } from "../../view/machines-copy.ts";
import type { Figure } from "./machine-figures.tsx";

/**
 * "Excludes forge-01 and lab-no-kvm". Each hostname stays whole where it
 * fits on its line, and a longer one wraps inside the line.
 */
function ExcludedMachines({ hostnames }: { hostnames: readonly string[] }) {
  const last = hostnames.length - 1;
  return (
    <View style={styles.excluded}>
      <Text style={text.caption}>{listCopy.excludes}</Text>
      {hostnames.map((hostname, index) => (
        <Fragment key={index}>
          {index > 0 && index === last ? (
            <Text style={text.caption}>{listCopy.and}</Text>
          ) : null}
          <Text style={[text.caption, styles.hostname]}>
            <Text style={text.mono}>{hostname}</Text>
            {last > 1 && index < last ? "," : ""}
          </Text>
        </Fragment>
      ))}
    </View>
  );
}

/** The cache hits figure of Machines. */
export function cacheHitsFigure(hits: CacheHits): Figure {
  if (hits.kind === "unknown") {
    return { label: listCopy.cacheHits, value: cacheCopy.unknown };
  }
  return {
    label: listCopy.cacheHits,
    value: hits.value,
    detail:
      hits.excluded.length > 0 ? (
        <ExcludedMachines hostnames={hits.excluded} />
      ) : undefined,
  };
}

const styles = StyleSheet.create({
  excluded: { flexDirection: "row", flexWrap: "wrap", columnGap: 4 },
  hostname: { maxWidth: "100%" },
});
