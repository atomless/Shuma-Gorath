# Generated Public-Content Site Standards And Generator Pattern Review

Date: 2026-03-30
Status: Active research driver

Related context:

- [`2026-03-30-contributor-generated-public-content-sim-site-review.md`](./2026-03-30-contributor-generated-public-content-sim-site-review.md)
- [`../plans/2026-03-30-contributor-generated-public-content-sim-site-plan.md`](../plans/2026-03-30-contributor-generated-public-content-sim-site-plan.md)
- [`../../scripts/build_site_surface_catalog.py`](../../scripts/build_site_surface_catalog.py)
- [`../../scripts/site_surface_catalog.py`](../../scripts/site_surface_catalog.py)
- [`../../src/runtime/sim_public.rs`](../../src/runtime/sim_public.rs)

## Purpose

Identify the cleanest implementation patterns and the most standards-aligned markup contract before building the contributor-generated `/sim/public/*` site.

This research specifically answers:

1. what page and archive structure best fits a deeply traversable date-entry site,
2. what semantic HTML patterns should be treated as the non-negotiable baseline,
3. what crawler-discoverability standards matter for internal links, sitemaps, canonical URLs, and `robots.txt`,
4. and what generator shape fits the repo’s existing tooling patterns without pulling markdown rendering into the runtime.

## Reference Shape Confirmed

The cited reference site, [dorian.fraser-moore.com](https://dorian.fraser-moore.com), is structurally useful for Shuma:

1. the effective homepage experience is an `All` or latest-posts feed rather than an about page,
2. the about content is separate,
3. chronology and pagination drive deep traversal,
4. and the site is heavily link-driven rather than interaction-driven.

That structure is a good match for adversary realism because it creates natural crawl depth and realistic “latest first, then archive” scraper behavior.

## Standards Findings

### 1. Semantic page structure should be explicit and sparse

The WHATWG HTML Standard treats:

1. `article` as a self-contained composition that is independently distributable or reusable,
2. `nav` as a section for major navigation blocks, not every cluster of links,
3. `main` as the page’s dominant content,
4. and `time` as the semantic timestamp element.

Implication for Shuma:

1. entry pages should be one primary `article` within `main`,
2. feed pages should render entry teasers as a list of `article` summaries,
3. timestamps should use `<time datetime=\"...\"></time>`,
4. and `nav` should be reserved for the site’s major section and pagination blocks.

Sources:

- WHATWG HTML Standard on `article`, `nav`, `main`, and `time`: [html.spec.whatwg.org](https://html.spec.whatwg.org/multipage/section.html), [html.spec.whatwg.org](https://html.spec.whatwg.org/multipage/), [html.spec.whatwg.org](https://html.spec.whatwg.org/multipage/links.html)

### 2. Crawlability depends on ordinary links, not UI cleverness

Google’s Search Central guidance is very blunt: links are reliably crawlable when they are real `<a>` elements with `href` attributes. That means pagination and archives should be ordinary linked pages, not JS-only state or button-driven incremental loading.

Implication for Shuma:

1. every feed page, section page, and entry page must be reachable via normal anchor links,
2. pagination must use ordinary URLs,
3. and no important traversal path should depend on JavaScript.

Sources:

- Google link crawlability guidance: [developers.google.com/search/docs/crawling-indexing/links-crawlable](https://developers.google.com/search/docs/crawling-indexing/links-crawlable)
- Google pagination guidance: [developers.google.com/search/docs/specialty/ecommerce/pagination-and-incremental-page-loading](https://developers.google.com/search/docs/specialty/ecommerce/pagination-and-incremental-page-loading)

### 3. Sitemap and canonical signals should reinforce, not replace, internal linking

Google says a sitemap is a hint, not a guarantee, and that properly linked pages may already be discoverable without it. Sitemaps still help on larger or more complex sites. Google also recommends canonical URLs be made clear with `rel=\"canonical\"`, and it prefers absolute canonical URLs in HTML. The Sitemap protocol also expects UTF-8 and absolute URLs.

Implication for Shuma:

1. internal linking remains primary,
2. `sitemap.xml` is still worth generating,
3. URLs in the sitemap should be canonical absolute URLs,
4. and each HTML page should emit one canonical URL in `<head>`.

Sources:

- Google sitemap overview: [developers.google.com/search/docs/crawling-indexing/sitemaps/overview](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview)
- Google sitemap build guidance: [developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)
- Google canonical guidance: [developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls)
- WHATWG canonical link type: [html.spec.whatwg.org](https://html.spec.whatwg.org/multipage/links.html)
- Sitemap protocol: [sitemaps.org](https://www.sitemaps.org/)

### 4. `robots.txt` is advisory and public

RFC 9309 makes two things especially important:

1. crawlers are automated clients,
2. and `robots.txt` is not authorization.

The RFC also explicitly warns that listing paths in `robots.txt` exposes them publicly.

Implication for Shuma:

1. `robots.txt` can advertise sitemap locations,
2. it must not be treated as a secret or security control,
3. and the generated site should not hide behind `robots.txt` logic.

Sources:

- RFC 9309: [rfc-editor.org/rfc/rfc9309](https://www.rfc-editor.org/rfc/rfc9309)

### 5. A standards-based feed is worth shipping

Because this site is chronology-heavy, an Atom feed is a natural fit. RFC 4287 defines the Atom Syndication Format, and RFC 5005 defines feed paging and archiving patterns.

Implication for Shuma:

1. generate at least one latest Atom feed for the site,
2. optionally generate section feeds,
3. and if archives are paginated, keep the HTML archives authoritative for traversal while the Atom feed remains a standards-based machine-oriented latest stream.

This is not required for human browsing, but it is a clean standards-compliant artifact for crawler realism and external tooling.

Sources:

- RFC 4287: [rfc-editor.org/rfc/rfc4287](https://www.rfc-editor.org/rfc/rfc4287)
- RFC 5005: [rfc-editor.org/rfc/rfc5005.html](https://www.rfc-editor.org/rfc/rfc5005.html)

### 6. Markdown should be parsed by a real CommonMark-conforming implementation

The CommonMark spec exists precisely to avoid ad hoc markdown rendering drift.

Implication for Shuma:

1. do not hand-roll markdown conversion with regexes,
2. choose a build-time parser that explicitly targets CommonMark behavior,
3. keep markdown rendering out of the request path,
4. and treat markdown-to-HTML conversion as generator work, not runtime work.

Source:

- CommonMark specification: [spec.commonmark.org](https://spec.commonmark.org/)

## Generator Pattern Recommendation

The cleanest repo-local implementation pattern is:

1. a small build-time CLI script,
2. a shared helper module,
3. deterministic output into a generated artifact directory,
4. and runtime serving of the generated artifact only.

That matches the existing style already used by:

1. [`scripts/build_site_surface_catalog.py`](../../scripts/build_site_surface_catalog.py)
2. [`scripts/site_surface_catalog.py`](../../scripts/site_surface_catalog.py)

Recommended generator shape:

1. scan allowlisted sources,
2. build a normalized entry model with title, date, excerpt, canonical path, source type, and HTML body,
3. emit:
   - root feed page,
   - section feed pages,
   - entry pages,
   - About page,
   - `sitemap.xml`,
   - `robots.txt` additions or companion sitemap references,
   - and `atom.xml`,
4. write a compact manifest for the runtime if useful,
5. and keep the runtime as a static file or manifest-backed responder rather than a renderer.

## Recommended Markup Contract

### Feed page

1. one `<main>`
2. one major heading
3. list of entry teasers, each as an `<article>`
4. each teaser includes:
   - linked heading,
   - `<time datetime>`
   - source-type label if useful
   - excerpt
5. pagination in a dedicated `<nav>`

### Entry page

1. one `<main>`
2. one `<article>`
3. entry header with title and `<time datetime>`
4. content body rendered from CommonMark
5. local context nav to section feed and root feed

### About page

1. separate URL
2. semantic article-like content from `README.md`
3. linked from primary navigation, not used as the root feed

## Conclusions

The best implementation is now clearer:

1. build-time generator, not runtime markdown rendering,
2. dated feed-root structure, not README-as-homepage,
3. semantic but sparse HTML,
4. crawlable linked pagination,
5. sitemap plus canonical plus Atom as supporting discoverability artifacts,
6. and a generator pattern that reuses the repo’s existing small Python CLI plus shared-helper style.
