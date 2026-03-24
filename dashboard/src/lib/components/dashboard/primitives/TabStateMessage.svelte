<script>
  export let tab = 'monitoring';
  export let status = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  const readMessage = (value) => String(value || '').trim();
  const readNoticeKind = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'success' || normalized === 'warning' || normalized === 'error') {
      return normalized;
    }
    return 'info';
  };

  $: tabStatus = status && typeof status === 'object' ? status : {};
  $: loading = tabStatus.loading === true;
  $: errorText = readMessage(tabStatus.error);
  $: empty = tabStatus.empty === true;
  $: statusMessage = readMessage(tabStatus.message);
  $: paneNoticeText = readMessage(noticeText);
  $: paneNoticeKind = readNoticeKind(noticeKind);
  $: message = loading
    ? (statusMessage || 'Loading...')
    : (errorText || (empty ? (statusMessage || 'No data.') : ''));
  $: stateKind = loading ? 'loading' : (errorText ? 'error' : (empty ? 'empty' : ''));
  $: className = stateKind ? `tab-state tab-state--${stateKind}` : 'tab-state';
</script>

<div
  class={className}
  data-tab-state={tab}
  role="status"
  aria-live="polite"
  hidden={!stateKind}
>{message}</div>

{#if paneNoticeText}
  <div
    class={`message ${paneNoticeKind}`}
    data-tab-notice={tab}
    role={paneNoticeKind === 'error' ? 'alert' : 'status'}
    aria-live={paneNoticeKind === 'error' ? 'assertive' : 'polite'}
  >{paneNoticeText}</div>
{/if}
