<script lang="ts">
  export let value: string = '';
  export let label: string = '';
</script>

<div class="editor-container">
  <label>{label}</label>
  <textarea
    {value}
    on:input={(e) => value = e.currentTarget.value}
    rows="10"
    spellcheck="false"
  />
</div>

<style>
  .editor-container {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  textarea {
    font-family: monospace;
    padding: 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    resize: vertical;
    min-height: 200px;
  }

  label {
    font-weight: bold;
  }
</style>
