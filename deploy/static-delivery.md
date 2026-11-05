# Static content delivery

Site and dashboard content use separate private GCS buckets in each
environment. Both buckets enforce PAP. A backend bucket is a load-balancer
wrapper, not another GCS bucket. Each origin has two wrappers:

| Wrapper name | Selected content | CDN policy |
| --- | --- | --- |
| `ciproxy-<env>-site`, `ciproxy-<env>-dashboard` | HTML, its aliases, and other static defaults | `enable_cdn=true`, `USE_ORIGIN_HEADERS`; publisher HTML `no-store` prevents caching |
| `ciproxy-<env>-site-assets`, `ciproxy-<env>-dashboard-assets` | Only the asset paths below | `enable_cdn=true`, `FORCE_CACHE_ALL` |

Private-bucket access requires `FORCE_CACHE_ALL` to cache content. Explicit
object metadata alone cannot enable private-bucket CDN caching.
[Google cache rules](https://docs.cloud.google.com/cdn/docs/caching#cacheable_content)

HTML wrappers keep private origin delivery and CDN controls enabled. They
do not force caching, have no signing keys, and honor stored `no-store` on
HTML and its bare/slash aliases. Both wrapper types disable negative
caching and stale serving. No HTML path selects a forced asset wrapper.
[Prevent caching](https://docs.cloud.google.com/cdn/docs/caching#non-cacheable_content_based_on_origin_headers)

## Lifetimes and publishing

Asset wrappers set these explicit values in seconds:

- `default_ttl=31536000` and `max_ttl=31536000`: one year at Cloud CDN.
- `client_ttl=86400`: response browser caching stays within one day.
- `serve_while_stale=0` and `negative_caching=false`.

FORCE_CACHE_ALL uses the default TTL even when origin metadata gives a
different lifetime. Its client TTL caps the response `max-age`. These
values satisfy the API's one-year limit and default/max/client ordering.
[Backend bucket API](https://docs.cloud.google.com/compute/docs/reference/rest/v1/backendBuckets)

Hashed bundles, images, and fonts can remain hot in CDN caches for a year.
Stable Pagefind files, sitemap XML, favicon, and Expo metadata share that
CDN policy. Every real deploy invalidates the whole URL map after edge
promotion. That refreshes stable names at CDN caches. Invalidation does not
clear a browser cache, which retains the existing one-day bound for assets.
CDN entries can be evicted before their configured lifetime.

The publisher still stores HTML `cacheControl=no-store` and asset
`cacheControl=public,max-age=86400,must-revalidate` in each multipart upload.
The asset backend's forced policy controls its effective CDN/client ages.
The publisher finishes all asset uploads before HTML and never deletes old
assets. Old pages can still request their earlier hashed assets.

## URL paths

| Host | Paths selecting its cached asset wrapper |
| --- | --- |
| Site | `/_astro/*`, `/pagefind/*`, `/favicon.svg`, `/sitemap-index.xml`, `/sitemap-0.xml` |
| Dashboard | `/_expo/static/*`, `/_expo/.routes.json`, `/assets/*` |

These are Google's exact paths and terminal `/*` prefix wildcards.
No extension glob or blanket `/docs/*` rule is used. The site public
directory currently contains only `favicon.svg`. Generated sitemaps have
exact root routes. Pagefind's JS, CSS, WASM, JSON, and search fragments use
its own prefix. Expo's static prefix also permits generated CSS/media,
without selecting the whole internal `/_expo/*` namespace.
[Path selection](https://docs.cloud.google.com/load-balancing/docs/url-map-concepts#order_of_operations),
[Expo reserved paths](https://docs.expo.dev/router/reference/reserved-paths/)

All other static paths use the HTML wrapper. `/` still rewrites to
`/index.html`. Docs, page routes, and publisher HTML aliases use that
uncached default. Dashboard `/api`, `/agent`, and `/hooks`, including their
children, keep the control-plane backend with CDN disabled. Dashboard
`/images` keeps its private signed-image backend and separate signing-key
flow. The download host keeps the release-server backend.

When build output or declared public assets change, review this path map
with the resulting exports and publisher aliases. The implementation's
bounded offline export proof covers current files. It does not call Google's
live URL-map validator or prove a cache hit.

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
