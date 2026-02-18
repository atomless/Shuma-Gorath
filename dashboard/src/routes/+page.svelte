<script>
  import { base } from '$app/paths';
  import { onMount } from 'svelte';
  import { mountDashboardRuntime } from '$lib/bridges/dashboard-runtime.js';
  import dashboardShell from '$lib/shell/dashboard-body.html?raw';

  const chartLiteSrc = `${base}/assets/vendor/chart-lite-1.0.0.min.js`;

  const pendingScripts = new Map();

  function ensureScript(src, dataKey) {
    if (pendingScripts.has(src)) {
      return pendingScripts.get(src);
    }

    if (document.querySelector(`script[${dataKey}="${src}"]`)) {
      return Promise.resolve();
    }

    const promise = new Promise((resolve, reject) => {
      const script = document.createElement('script');
      script.src = src;
      script.async = false;
      script.setAttribute(dataKey, src);
      script.addEventListener('load', () => resolve(), { once: true });
      script.addEventListener('error', () => reject(new Error(`Failed to load ${src}`)), {
        once: true
      });
      document.head.appendChild(script);
    });

    pendingScripts.set(src, promise);
    return promise;
  }

  onMount(async () => {
    await ensureScript(chartLiteSrc, 'data-shuma-runtime-script');
    await mountDashboardRuntime();
  });
</script>

<svelte:head>
  <title>Shuma-Gorath Dashboard</title>
</svelte:head>

{@html dashboardShell}
