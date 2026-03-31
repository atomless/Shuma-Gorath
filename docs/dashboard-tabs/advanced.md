# Dashboard Tab: Advanced

Route: `#advanced`  
Component: [`dashboard/src/lib/components/dashboard/AdvancedTab.svelte`](../../dashboard/src/lib/components/dashboard/AdvancedTab.svelte)

Purpose:

- Provide full runtime variable inventory and low-level JSON patch editing.

Panels:

- `Runtime Variable Inventory`:
  - grouped snapshot of runtime config variables,
  - includes both admin-writable variables and read-only runtime-visible knobs,
  - rows highlight admin-writable variables,
  - per-variable meaning text from status variable metadata.
- `Export Config JSON` helper:
  - downloads and optionally copies the current Advanced JSON payload.
- `Advanced Config JSON`:
  - editable JSON object patch seeded from writable template paths,
  - always-on line numbers,
  - parse diagnostics and server validation diagnostics,
  - line/column issue reporting where available.

Validation/save flow:

- Client parse check must pass (valid JSON object).
- Server validation (`POST /shuma/admin/config/validate`) must pass.
- Save writes patch to `POST /shuma/admin/config`.

Policy note:

- Advanced JSON must remain in parity with all non-env-only writable admin config keys.
- Admin writability must not be confused with controller eligibility. Advanced exposes the broader operator-writable surface, which now sits under the canonical mutability rings:
  - `never`
  - `manual_only`
  - `controller_tunable`
- Later controller-explanation work must consume that canonical classification rather than inventing a second local grouping inside Advanced.
