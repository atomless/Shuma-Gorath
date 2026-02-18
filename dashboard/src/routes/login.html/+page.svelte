<script>
  import { onMount } from 'svelte';
  import { mountLoginRuntime } from '$lib/bridges/login-runtime.js';
  import loginShell from '$lib/shell/login-body.html?raw';

  onMount(async () => {
    await mountLoginRuntime();
  });
</script>

<svelte:head>
  <title>Shuma-Gorath Dashboard Login</title>
</svelte:head>

{@html loginShell}
