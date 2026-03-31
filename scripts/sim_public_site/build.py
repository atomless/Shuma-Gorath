"""Build the contributor-generated root-hosted public site artifact."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
import html
import json
from pathlib import Path
import re
import shutil
import subprocess
from typing import Iterable

FRESHNESS_FILENAME = "freshness.json"
MANIFEST_FILENAME = "manifest.json"
SITE_CONTENT_DIRNAME = "site"
LISTING_PAGE_SIZE = 20

SECTION_ORDER = ("research", "plans", "work")
SECTION_LABELS = {
    "research": "Research",
    "plans": "Plans",
    "work": "Completed Work",
}
DATE_FILENAME_PATTERN = re.compile(r"^(?P<date>\d{4}-\d{2}-\d{2})-(?P<slug>.+)\.md$")
COMPLETION_DATE_PATTERN = re.compile(r"^## .*\((\d{4}-\d{2}-\d{2})\)\s*$")
COMPLETION_ENTRY_PATTERN = re.compile(r"^### (.+)$")


@dataclass(frozen=True)
class SectionConfig:
    key: str
    output_prefix: str
    source_glob: str | None = None
    source_file: str | None = None


@dataclass(frozen=True)
class Entry:
    section: str
    title: str
    date_iso: str
    slug: str
    route: str
    output_path: Path
    markdown: str
    excerpt: str
    source_path: str
    html_body: str


def build_site(
    repo_root: Path,
    artifact_root: Path,
    corpus_config_path: Path,
    site_url: str,
) -> dict[str, object]:
    corpus = load_corpus_config(corpus_config_path)
    root_prefix = normalize_root_prefix(corpus["site"]["root_prefix"])
    sections = load_sections(corpus)
    rendered_entries = render_entries(repo_root, root_prefix, sections)
    about_html = render_markdown((repo_root / corpus["site"]["about_source"]).read_text(encoding="utf-8"))

    site_root = artifact_root / SITE_CONTENT_DIRNAME
    reset_artifact_root(artifact_root)
    site_root.mkdir(parents=True, exist_ok=True)

    generated_at = timestamp_utc()
    page_routes: list[str] = []
    for entry in rendered_entries:
        write_html(site_root / entry.output_path, render_entry_page(entry, site_url, root_prefix))
    entry_routes = [entry.route for entry in rendered_entries]

    for output_path, route, html_document in render_listing_pages(
        entries=rendered_entries,
        site_url=site_url,
        root_prefix=root_prefix,
        section_key=None,
    ):
        write_html(site_root / output_path, html_document)
        page_routes.append(route)
    about_route = f"{root_prefix}/about/"
    write_html(
        site_root / "about" / "index.html",
        render_about_page(about_html, site_url, root_prefix),
    )
    page_routes.append(about_route)
    (site_root / "atom.xml").write_text(
        f"{render_atom_feed(rendered_entries, site_url, root_prefix)}\n",
        encoding="utf-8",
    )
    for section_key in SECTION_ORDER:
        section_entries = [entry for entry in rendered_entries if entry.section == section_key]
        for output_path, route, html_document in render_listing_pages(
            entries=section_entries,
            site_url=site_url,
            root_prefix=root_prefix,
            section_key=section_key,
        ):
            write_html(site_root / output_path, html_document)
            page_routes.append(route)

    write_text(
        site_root / "robots.txt",
        render_robots_txt(site_url, root_prefix),
    )
    write_text(
        site_root / "sitemap.xml",
        render_sitemap_index(site_url, root_prefix),
    )
    write_text(
        site_root / "sitemaps" / "pages.xml",
        render_urlset(page_routes, site_url),
    )
    write_text(
        site_root / "sitemaps" / "entries.xml",
        render_urlset(entry_routes, site_url),
    )

    manifest = {
        "schema": "shuma.sim_public_site.v1",
        "generated_at_utc": generated_at,
        "site_url": site_url,
        "root_path": f"{root_prefix}/",
        "about_path": f"{root_prefix}/about/",
        "page_routes": page_routes,
        "entries": [
            {
                "section": entry.section,
                "title": entry.title,
                "date": entry.date_iso,
                "route": entry.route,
                "source_path": entry.source_path,
            }
            for entry in rendered_entries
        ],
    }
    freshness = {
        "generated_at_utc": generated_at,
        "source_paths": freshness_source_paths(
            repo_root=repo_root,
            corpus_config_path=corpus_config_path,
            about_source=str(corpus["site"]["about_source"]),
            rendered_entries=rendered_entries,
        ),
    }
    write_json(artifact_root / MANIFEST_FILENAME, manifest)
    write_json(artifact_root / FRESHNESS_FILENAME, freshness)
    return manifest


def build_site_if_stale(
    repo_root: Path,
    artifact_root: Path,
    corpus_config_path: Path,
    site_url: str,
    if_stale_hours: int,
) -> dict[str, object] | None:
    if if_stale_hours < 0:
        raise ValueError("if_stale_hours must be zero or greater")
    if not refresh_required(repo_root, artifact_root, if_stale_hours):
        return None
    return build_site(
        repo_root=repo_root,
        artifact_root=artifact_root,
        corpus_config_path=corpus_config_path,
        site_url=site_url,
    )


def refresh_required(repo_root: Path, artifact_root: Path, if_stale_hours: int) -> bool:
    manifest_path = artifact_root / MANIFEST_FILENAME
    freshness_path = artifact_root / FRESHNESS_FILENAME
    site_root = artifact_root / SITE_CONTENT_DIRNAME
    if not manifest_path.is_file() or not freshness_path.is_file() or not site_root.is_dir():
        return True

    freshness = load_json_file(freshness_path)
    generated_at_utc = str(freshness.get("generated_at_utc") or "").strip()
    generated_at = parse_utc_timestamp(generated_at_utc)
    if generated_at is None:
        return True
    if datetime.now(timezone.utc) - generated_at > timedelta(hours=if_stale_hours):
        return True

    source_paths = freshness.get("source_paths")
    if not isinstance(source_paths, list) or not source_paths:
        return True
    for relative_path in source_paths:
        source_path = repo_root / str(relative_path)
        if not source_path.is_file():
            return True
        if datetime.fromtimestamp(source_path.stat().st_mtime, timezone.utc) > generated_at:
            return True
    return False


def freshness_source_paths(
    *,
    repo_root: Path,
    corpus_config_path: Path,
    about_source: str,
    rendered_entries: Iterable[Entry],
) -> list[str]:
    source_paths = {
        about_source,
        *(entry.source_path for entry in rendered_entries),
        path_reference(repo_root, corpus_config_path),
        *generator_source_paths(repo_root),
    }
    return sorted(source_paths)


def generator_source_paths(repo_root: Path) -> list[str]:
    build_path = Path(__file__).resolve()
    return [
        path_reference(repo_root, build_path.parents[1] / "build_sim_public_site.py"),
        path_reference(repo_root, build_path.parent / "__init__.py"),
        path_reference(repo_root, build_path),
        path_reference(repo_root, build_path.with_name("render_markdown.mjs")),
    ]


def path_reference(repo_root: Path, path: Path) -> str:
    resolved = path.resolve()
    try:
        return resolved.relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return resolved.as_posix()


def load_json_file(path: Path) -> dict[str, object]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}
    if not isinstance(payload, dict):
        return {}
    return payload


def parse_utc_timestamp(value: str) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def load_corpus_config(path: Path) -> dict[str, object]:
    data: dict[str, object] = {}
    current_table: dict[str, object] | None = None
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section_name = line[1:-1].strip()
            current_table = ensure_table(data, section_name)
            continue
        if "=" not in line or current_table is None:
            raise ValueError(f"unsupported corpus config line: {raw_line}")
        key, value = line.split("=", 1)
        current_table[key.strip()] = parse_toml_string(value.strip())
    return data


def ensure_table(root: dict[str, object], dotted_name: str) -> dict[str, object]:
    current: dict[str, object] = root
    for part in dotted_name.split("."):
        existing = current.get(part)
        if isinstance(existing, dict):
            current = existing
            continue
        next_table: dict[str, object] = {}
        current[part] = next_table
        current = next_table
    return current


def parse_toml_string(raw_value: str) -> str:
    if len(raw_value) >= 2 and raw_value.startswith('"') and raw_value.endswith('"'):
        return raw_value[1:-1]
    raise ValueError(f"unsupported corpus config value: {raw_value}")


def load_sections(corpus: dict[str, object]) -> list[SectionConfig]:
    sections_raw = corpus.get("sections", {})
    if not isinstance(sections_raw, dict):
        raise ValueError("corpus sections must be a table")
    sections: list[SectionConfig] = []
    for key in SECTION_ORDER:
        raw = sections_raw.get(key)
        if not isinstance(raw, dict):
            continue
        sections.append(
            SectionConfig(
                key=key,
                output_prefix=str(raw["output_prefix"]),
                source_glob=raw.get("source_glob"),
                source_file=raw.get("source_file"),
            )
        )
    return sections


def render_entries(repo_root: Path, root_prefix: str, sections: Iterable[SectionConfig]) -> list[Entry]:
    entries: list[Entry] = []
    for section in sections:
        if section.source_glob:
            entries.extend(load_markdown_entries(repo_root, root_prefix, section))
        elif section.source_file:
            entries.extend(load_completed_work_entries(repo_root, root_prefix, section))
    entries.sort(key=lambda entry: (entry.date_iso, entry.title.lower()), reverse=True)
    return entries


def load_markdown_entries(repo_root: Path, root_prefix: str, section: SectionConfig) -> list[Entry]:
    entries: list[Entry] = []
    for path in sorted(repo_root.glob(section.source_glob or "")):
        match = DATE_FILENAME_PATTERN.match(path.name)
        if not match:
            continue
        date_iso = match.group("date")
        slug = f"{date_iso}-{slugify(match.group('slug'))}"
        markdown = path.read_text(encoding="utf-8")
        title = markdown_title(markdown) or humanize_slug(match.group("slug"))
        route = f"{root_prefix}/{section.output_prefix}/{slug}/"
        entries.append(
            Entry(
                section=section.key,
                title=title,
                date_iso=date_iso,
                slug=slug,
                route=route,
                output_path=Path(section.output_prefix) / slug / "index.html",
                markdown=markdown,
                excerpt=excerpt_from_markdown(markdown),
                source_path=path.relative_to(repo_root).as_posix(),
                html_body=render_markdown(markdown),
            )
        )
    return entries


def load_completed_work_entries(repo_root: Path, root_prefix: str, section: SectionConfig) -> list[Entry]:
    source_path = repo_root / (section.source_file or "")
    lines = source_path.read_text(encoding="utf-8").splitlines()
    current_date: str | None = None
    current_title: str | None = None
    current_body: list[str] = []
    entries: list[Entry] = []

    def flush_current() -> None:
        nonlocal current_title, current_body
        if not current_date or not current_title:
            current_title = None
            current_body = []
            return
        slug = f"{current_date}-{slugify(current_title)}"
        markdown = "\n".join(current_body).strip()
        route = f"{root_prefix}/{section.output_prefix}/{slug}/"
        entries.append(
            Entry(
                section=section.key,
                title=current_title,
                date_iso=current_date,
                slug=slug,
                route=route,
                output_path=Path(section.output_prefix) / slug / "index.html",
                markdown=markdown,
                excerpt=excerpt_from_markdown(markdown),
                source_path=source_path.relative_to(repo_root).as_posix(),
                html_body=render_markdown(markdown),
            )
        )
        current_title = None
        current_body = []

    for line in lines:
        date_match = COMPLETION_DATE_PATTERN.match(line)
        if date_match:
            flush_current()
            current_date = date_match.group(1)
            continue
        title_match = COMPLETION_ENTRY_PATTERN.match(line)
        if title_match:
            flush_current()
            current_title = title_match.group(1).strip()
            continue
        if current_title is not None:
            current_body.append(line)
    flush_current()
    return entries


def normalize_root_prefix(value: str) -> str:
    stripped = "/" + value.strip().strip("/")
    return stripped.rstrip("/")


def render_markdown(markdown_text: str) -> str:
    script_path = Path(__file__).with_name("render_markdown.mjs")
    result = subprocess.run(
        ["node", str(script_path)],
        input=markdown_text,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "markdown renderer failed")
    return result.stdout.strip()


def markdown_title(markdown_text: str) -> str | None:
    for line in markdown_text.splitlines():
        stripped = line.strip()
        if stripped.startswith("# "):
            return stripped[2:].strip()
    return None


def excerpt_from_markdown(markdown_text: str) -> str:
    for line in markdown_text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        stripped = stripped.removeprefix("- [x] ").removeprefix("- ").strip()
        stripped = re.sub(r"`([^`]+)`", r"\1", stripped)
        return stripped[:220]
    return ""


def render_listing_pages(
    *,
    entries: list[Entry],
    site_url: str,
    root_prefix: str,
    section_key: str | None,
) -> list[tuple[Path, str, str]]:
    pages = paginate_entries(entries)
    total_pages = max(1, len(pages))
    rendered: list[tuple[Path, str, str]] = []
    for index, page_entries in enumerate(pages, start=1):
        route = listing_route(root_prefix, section_key, index)
        output_path = listing_output_path(section_key, index)
        body_html = render_entry_listing(page_entries)
        pagination_html = render_pagination_nav(root_prefix, section_key, index, total_pages)
        if pagination_html:
            body_html = f"{body_html}{pagination_html}"
        if section_key is None:
            title = "Latest" if index == 1 else f"Latest Page {index}"
            heading = "Latest"
            lead_html = (
                "<p>A dated public feed of research, plans, and shipped work.</p>"
                if index == 1
                else f"<p>Older dated entries, page {index}.</p>"
            )
        else:
            label = SECTION_LABELS[section_key]
            title = label if index == 1 else f"{label} Page {index}"
            heading = label
            lead_html = (
                f"<p>{html.escape(label)} entries.</p>"
                if index == 1
                else f"<p>{html.escape(label)} archive page {index}.</p>"
            )
        rendered.append(
            (
                output_path,
                route,
                render_document(
                    title=title,
                    canonical_url=f"{site_url}{route}",
                    root_prefix=root_prefix,
                    heading=heading,
                    lead_html=lead_html,
                    body_html=body_html,
                ),
            )
        )
    return rendered


def paginate_entries(entries: list[Entry]) -> list[list[Entry]]:
    if not entries:
        return [[]]
    return [
        entries[index : index + LISTING_PAGE_SIZE]
        for index in range(0, len(entries), LISTING_PAGE_SIZE)
    ]


def listing_route(root_prefix: str, section_key: str | None, page_number: int) -> str:
    base = f"{root_prefix}/" if section_key is None else f"{root_prefix}/{section_key}/"
    if page_number <= 1:
        return base
    return f"{base}page/{page_number}/"


def listing_output_path(section_key: str | None, page_number: int) -> Path:
    if section_key is None:
        if page_number <= 1:
            return Path("index.html")
        return Path("page") / str(page_number) / "index.html"
    if page_number <= 1:
        return Path(section_key) / "index.html"
    return Path(section_key) / "page" / str(page_number) / "index.html"


def render_pagination_nav(
    root_prefix: str,
    section_key: str | None,
    page_number: int,
    total_pages: int,
) -> str:
    if total_pages <= 1:
        return ""
    links: list[str] = []
    if page_number > 1:
        links.append(
            f'<a rel="prev" href="{html.escape(listing_route(root_prefix, section_key, page_number - 1))}">Newer</a>'
        )
    if page_number < total_pages:
        links.append(
            f'<a rel="next" href="{html.escape(listing_route(root_prefix, section_key, page_number + 1))}">Older</a>'
        )
    page_links = " ".join(
        (
            f"<strong>{index}</strong>"
            if index == page_number
            else f'<a href="{html.escape(listing_route(root_prefix, section_key, index))}">{index}</a>'
        )
        for index in range(1, total_pages + 1)
    )
    return (
        '<nav aria-label="Pagination">'
        f"<p>{' | '.join(links)}</p>"
        f"<p>{page_links}</p>"
        "</nav>"
    )


def render_atom_feed(entries: list[Entry], site_url: str, root_prefix: str) -> str:
    updated = entries[0].date_iso if entries else timestamp_utc()
    entry_xml = "".join(
        (
            "<entry>"
            f"<title>{html.escape(entry.title)}</title>"
            f"<id>{html.escape(site_url + entry.route)}</id>"
            f"<link href=\"{html.escape(site_url + entry.route)}\" />"
            f"<updated>{entry.date_iso}T00:00:00Z</updated>"
            f"<summary>{html.escape(entry.excerpt)}</summary>"
            "</entry>"
        )
        for entry in entries
    )
    return (
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>"
        "<feed xmlns=\"http://www.w3.org/2005/Atom\">"
        "<title>Shuma Contributor Public Site</title>"
        f"<id>{html.escape(site_url + root_prefix + '/')}</id>"
        f"<link href=\"{html.escape(site_url + root_prefix + '/atom.xml')}\" rel=\"self\" />"
        f"<link href=\"{html.escape(site_url + root_prefix + '/')}\" />"
        f"<updated>{updated}T00:00:00Z</updated>"
        f"{entry_xml}"
        "</feed>"
    )


def render_robots_txt(site_url: str, root_prefix: str) -> str:
    return (
        "User-agent: *\n"
        "Allow: /\n"
        f"Sitemap: {site_url}{root_prefix}/sitemap.xml\n"
    )


def render_sitemap_index(site_url: str, root_prefix: str) -> str:
    return (
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>"
        "<sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"
        f"<sitemap><loc>{html.escape(site_url + root_prefix + '/sitemaps/pages.xml')}</loc></sitemap>"
        f"<sitemap><loc>{html.escape(site_url + root_prefix + '/sitemaps/entries.xml')}</loc></sitemap>"
        "</sitemapindex>"
    )


def render_urlset(routes: list[str], site_url: str) -> str:
    entries = "".join(
        f"<url><loc>{html.escape(site_url + route)}</loc></url>" for route in routes
    )
    return (
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>"
        "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"
        f"{entries}"
        "</urlset>"
    )


def render_about_page(about_html: str, site_url: str, root_prefix: str) -> str:
    return render_document(
        title="About",
        canonical_url=f"{site_url}{root_prefix}/about/",
        root_prefix=root_prefix,
        heading=None,
        lead_html="",
        body_html=f"<article>{about_html}</article>",
    )


def render_entry_page(entry: Entry, site_url: str, root_prefix: str) -> str:
    body_html = (
        f"<article>"
        f"<header><p><time datetime=\"{entry.date_iso}\">{entry.date_iso}</time></p>"
        f"<h1>{html.escape(entry.title)}</h1></header>"
        f"{entry.html_body}"
        f"</article>"
    )
    return render_document(
        title=entry.title,
        canonical_url=f"{site_url}{entry.route}",
        root_prefix=root_prefix,
        heading=None,
        lead_html="",
        body_html=body_html,
    )


def render_document(
    *,
    title: str,
    canonical_url: str,
    root_prefix: str,
    heading: str | None,
    lead_html: str,
    body_html: str,
) -> str:
    nav_html = render_nav(root_prefix)
    header_html = f"<h1>{html.escape(heading)}</h1>" if heading else ""
    return (
        "<!doctype html>"
        "<html lang=\"en\">"
        "<head>"
        "<meta charset=\"utf-8\">"
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
        f"<title>{html.escape(title)}</title>"
        f"<link rel=\"canonical\" href=\"{html.escape(canonical_url)}\">"
        "</head>"
        "<body>"
        "<header>"
        f"<nav>{nav_html}</nav>"
        f"{header_html}"
        f"{lead_html}"
        "</header>"
        f"<main>{body_html}</main>"
        "<footer><p>Generated contributor public-content site.</p></footer>"
        "</body>"
        "</html>"
    )


def render_nav(root_prefix: str) -> str:
    links = [
        (f"{root_prefix}/", "Latest"),
        (f"{root_prefix}/about/", "About"),
        (f"{root_prefix}/research/", "Research"),
        (f"{root_prefix}/plans/", "Plans"),
        (f"{root_prefix}/work/", "Completed Work"),
    ]
    return " | ".join(
        f"<a href=\"{html.escape(href)}\">{html.escape(label)}</a>" for href, label in links
    )


def render_entry_listing(entries: list[Entry]) -> str:
    articles = []
    for entry in entries:
        excerpt_html = (
            f"<p>{html.escape(entry.excerpt)}</p>" if entry.excerpt else ""
        )
        articles.append(
            "<article>"
            f"<p><time datetime=\"{entry.date_iso}\">{entry.date_iso}</time></p>"
            f"<h2><a href=\"{html.escape(entry.route)}\">{html.escape(entry.title)}</a></h2>"
            f"{excerpt_html}"
            "</article>"
        )
    return "".join(articles)


def reset_artifact_root(artifact_root: Path) -> None:
    if artifact_root.exists():
        shutil.rmtree(artifact_root)
    artifact_root.mkdir(parents=True, exist_ok=True)


def write_html(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"{content}\n", encoding="utf-8")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"{json.dumps(payload, indent=2, sort_keys=True)}\n", encoding="utf-8")


def timestamp_utc() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def slugify(value: str) -> str:
    lowered = value.strip().lower()
    lowered = re.sub(r"[^a-z0-9]+", "-", lowered)
    lowered = lowered.strip("-")
    return lowered or "entry"


def humanize_slug(value: str) -> str:
    return " ".join(part.capitalize() for part in slugify(value).split("-"))
