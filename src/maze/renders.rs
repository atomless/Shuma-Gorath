pub(crate) struct AdvancedMazeLink {
    pub href: String,
    pub text: String,
    pub description: String,
    pub pow_difficulty: Option<u8>,
}

pub(crate) struct AdvancedMazeRenderOptions {
    pub title: String,
    pub breadcrumb: String,
    pub paragraphs: Vec<String>,
    pub links: Vec<AdvancedMazeLink>,
    pub bootstrap_json: String,
    pub variant_layout: u8,
    pub variant_palette: u8,
    pub style_tier: MazeStyleTier,
    pub style_sheet_url: Option<String>,
    pub script_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeStyleTier {
    Full,
    Lite,
    Machine,
}

fn palette(variant_palette: u8) -> (&'static str, &'static str, &'static str, &'static str) {
    match variant_palette % 3 {
        0 => ("#0f172a", "#e2e8f0", "#38bdf8", "#f8fafc"),
        1 => ("#1f2937", "#fef3c7", "#f59e0b", "#fffbeb"),
        _ => ("#1e293b", "#dcfce7", "#22c55e", "#f0fdf4"),
    }
}

fn escape_script_json(value: &str) -> String {
    value.replace("</", "<\\/")
}

pub(crate) fn generate_polymorphic_maze_page(options: &AdvancedMazeRenderOptions) -> String {
    let (_header_bg, _header_fg, _accent, _panel_bg) = palette(options.variant_palette);
    let layout_class = match options.variant_layout % 3 {
        0 => "layout-grid",
        1 => "layout-stacked",
        _ => "layout-ribbon",
    };
    let style_class = match options.style_tier {
        MazeStyleTier::Full => "style-full",
        MazeStyleTier::Lite => "style-lite",
        MazeStyleTier::Machine => "style-machine",
    };
    let mut head_assets = String::new();
    if let Some(url) = &options.style_sheet_url {
        head_assets.push_str(format!(r#"<link rel="stylesheet" href="{}">"#, url).as_str());
    }
    head_assets
        .push_str(format!(r#"<script defer src="{}"></script>"#, options.script_url).as_str());

    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <meta name="robots" content="noindex,nofollow,noarchive">
    {}
</head>
<body class="{}">
    <div class="wrap {} {}">
        <header>
            <h1>{}</h1>
            <div class="crumb">{}</div>
        </header>
        <div class="content">
"#,
        options.title,
        head_assets,
        style_class,
        layout_class,
        style_class,
        options.title,
        options.breadcrumb
    );

    for paragraph in &options.paragraphs {
        html.push_str(&format!(
            r#"            <p class="description">{}</p>
"#,
            paragraph
        ));
    }

    html.push_str(
        r#"            <div class="nav-grid" id="maze-nav-grid">
"#,
    );

    for link in &options.links {
        let pow_hint = link
            .pow_difficulty
            .map(|difficulty| {
                format!(
                    r#"<div class="pow-hint">Deep-traversal proof required ({} bits)</div>"#,
                    difficulty
                )
            })
            .unwrap_or_default();
        let pow_attr = link
            .pow_difficulty
            .map(|difficulty| format!(r#" data-pow-difficulty="{}""#, difficulty))
            .unwrap_or_default();
        html.push_str(&format!(
            r#"                <a href="{}" class="nav-card"{} data-link-kind="maze">
                    <h3>{}</h3>
                    <p>{}</p>
                    {}
                    <div class="arrow">Continue →</div>
                </a>
"#,
            link.href, pow_attr, link.text, link.description, pow_hint
        ));
    }

    html.push_str(
        r#"            </div>
        </div>
"#,
    );
    html.push_str(
        r#"    </div>
    <script id="maze-bootstrap" type="application/json">"#,
    );
    html.push_str(escape_script_json(options.bootstrap_json.as_str()).as_str());
    html.push_str(
        r#"</script>
</body>
</html>"#,
    );

    html
}
