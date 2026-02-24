<script>
  export let colspan = 1;
  export let text = 'No data.';
</script>

<tr>
  <td colspan={colspan} class="table-empty-cell">
    <slot>{text}</slot>
  </td>
</tr>
