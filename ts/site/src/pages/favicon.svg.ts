// The favicon of the marketing pages and the docs, built from the brand mark
// and the ink token. Starlight and the marketing layout link to
// `/favicon.svg`.

import type { APIRoute } from "astro";

import { brandFaviconSvg } from "../data/brand.ts";

export const GET: APIRoute = () =>
  new Response(brandFaviconSvg(), {
    headers: { "Content-Type": "image/svg+xml" },
  });
