// The width class of the window: a phone, a narrow window where the sidebar
// becomes an icon rail, or a wide window. The static render has no window,
// so the first client render uses the wide class too, and the real class
// follows after the page mounts. The server HTML and the first client render
// then agree, as hydration needs.

import { useEffect, useState } from "react";
import { useWindowDimensions } from "react-native";

export type Viewport = "mobile" | "narrow" | "wide";

export const breakpoints = { mobile: 640, wide: 961 } as const;

export function viewportFor(width: number): Viewport {
  if (width < breakpoints.mobile) return "mobile";
  if (width < breakpoints.wide) return "narrow";
  return "wide";
}

/**
 * Whether the page has mounted in the browser. Content that depends on the
 * window or the query waits for it, so that the first client render equals
 * the HTML of the export.
 */
export function useMounted(): boolean {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  return mounted;
}

export function useViewport(): Viewport {
  const { width } = useWindowDimensions();
  return useMounted() ? viewportFor(width) : "wide";
}
