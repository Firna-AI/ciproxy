# Static content delivery

Site and dashboard content use separate private GCS buckets in each
environment. Both buckets enforce PAP. A backend bucket is a load-balancer
wrapper, not another GCS bucket. Each origin has three wrappers:

| Wrapper name | Selected content | CDN policy |
| --- | --- | --- |
| `ciproxy-<env>-site`, `ciproxy-<env>-dashboard` | HTML, its aliases, and other static defaults | `enable_cdn=true`, `USE_ORIGIN_HEADERS`; publisher HTML `no-store` prevents caching |
| `ciproxy-<env>-site-assets`, `ciproxy-<env>-dashboard-assets` | Generated immutable hash paths | `enable_cdn=true`, `FORCE_CACHE_ALL`, one-year CDN / one-day client |
| `ciproxy-<env>-site-mutable-assets`, `ciproxy-<env>-dashboard-mutable-assets` | Mutable static paths | `enable_cdn=true`, `FORCE_CACHE_ALL`, five-minute CDN/client |

Private-bucket access requires `FORCE_CACHE_ALL` to cache content. Explicit
object metadata alone cannot enable private-bucket CDN caching.
[Google cache rules](https://docs.cloud.google.com/cdn/docs/caching#cacheability)

HTML wrappers keep private origin delivery and CDN controls enabled. They
do not force caching, have no signing keys, and honor stored `no-store` on
HTML and its bare/slash aliases. All wrapper types disable negative
caching and stale serving. No HTML path selects a forced asset wrapper.
[Prevent caching](https://docs.cloud.google.com/cdn/docs/caching#non-cacheable_content)

## Lifetimes and publishing

Immutable wrappers set these explicit values in seconds:

- `default_ttl=31536000` and `max_ttl=31536000`: one year at Cloud CDN.
- `client_ttl=86400`: response browser caching stays within one day.
- `serve_while_stale=0` and `negative_caching=false`.

Mutable wrappers set `default_ttl=300`, `max_ttl=300`, and `client_ttl=300`.
They also disable stale serving and negative caching. A stale mutable entry
expires within five minutes even if a deployment stops after publication
and never invalidates. Browsers also receive at most five minutes.

FORCE_CACHE_ALL uses the default TTL even when origin metadata gives a
different lifetime. Its client TTL caps the response `max-age`. These
values satisfy the API's one-year limit and default/max/client ordering.
[Backend bucket API](https://docs.cloud.google.com/compute/docs/reference/rest/v1/backendBuckets)

Hashed bundles, images, and fonts can remain hot in CDN caches for a year.
Stable Pagefind files, sitemap XML, favicon, and Expo metadata use the short
wrapper. Pagefind's hashed `pf_meta` also uses that safe short wrapper
because it shares a directory with mutable files. Every real deploy still
invalidates the whole URL map after edge promotion. Invalidation does not
clear a browser cache. Immutable assets retain the one-day browser bound.
CDN entries can be evicted before their configured lifetime.

The publisher still stores HTML `cacheControl=no-store` and asset
`cacheControl=public,max-age=86400,must-revalidate` in each multipart upload.
The asset backend's forced policy controls its effective CDN/client ages.
The publisher finishes all asset uploads before HTML and never deletes old
assets. Old pages can still request their earlier hashed assets.

## URL paths

| Host | One-year immutable wrapper | Five-minute mutable wrapper |
| --- | --- | --- |
| Site | `/_astro/*`, `/pagefind/index/*`, `/pagefind/fragment/*` | `/pagefind/*`, `/favicon.svg`, `/sitemap-index.xml`, `/sitemap-0.xml` |
| Dashboard | `/_expo/static/*`, `/assets/*` | `/_expo/.routes.json` |

These are Google's exact paths and terminal `/*` prefix wildcards.
No extension glob or blanket `/docs/*` rule is used. The site public
directory currently contains only `favicon.svg`. Generated sitemaps have
exact root routes. Pagefind's more specific hashed index/fragment prefixes
override its mutable parent. Stable JS, CSS, WASM, and entry JSON stay short.
Expo's static prefix also permits generated hashed CSS/media,
without selecting the whole internal `/_expo/*` namespace.
[Path selection](https://docs.cloud.google.com/load-balancing/docs/url-map-concepts#order_of_operations),
[Expo reserved paths](https://docs.expo.dev/router/reference/reserved-paths/)

All other static paths use the HTML wrapper. `/` still rewrites to
`/index.html`. Docs, page routes, and publisher HTML aliases use that
uncached default. Dashboard `/api`, `/agent`, and `/hooks`, including their
children, keep the control-plane backend with CDN disabled. Dashboard
`/images` keeps its private signed-image backend and separate signing-key
flow. The download host keeps the release-server backend.

The export-inventory test reads actual builds, publisher aliases, and the
mocked planned URL map. Copied public files are mutable by source provenance,
even if their names resemble hashes. Generated filename/hash classification
is independent of these routes. A stable file inside a one-year prefix fails.
Build both exports first as the README requires. Review the route map when
public assets or output names change. This test does not call Google's live
URL-map validator or prove a cache hit.

## Private origin access

Each GCS bucket receives exactly one Object Viewer grant for
`service-PROJECT_NUMBER@https-lb.iam.gserviceaccount.com`. The grant iterates
the cached asset wrappers, which must exist first. HTML and asset wrappers
share that single reader. The signed-image cache-fill identity is separate.

Static-only bucket policy setters, PAP, and domain-restricted sharing stay
as [deployment identities](identities.md#private-static-origins) describes.
No new provider or credential is needed. A first real service-agent grant
that fails organization policy stops this path and returns item 76 to the
user. Do not widen that policy automatically.
