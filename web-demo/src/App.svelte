<script lang="js">
  import CodeEditor from "./lib/CodeEditor.svelte";
  import init, {
    analyze_bril_program,
    run_datalog_analysis,
  } from "../datalog_wasm/pkg/datalog_wasm";

  // Initialize the Wasm module
  init();

  let brilCode = `{
  "functions": [
    {
      "instrs": [
        {
          "dest": "x",
          "op": "const",
          "type": "int",
          "value": 3
        },
        {
          "dest": "y",
          "op": "const",
          "type": "int",
          "value": 5
        },
        {
          "args": [
            "x",
            "y"
          ],
          "dest": "y",
          "op": "add",
          "type": "int"
        },
        {
          "args": [
            "y"
          ],
          "op": "print"
        },
        {
          "dest": "x",
          "op": "const",
          "type": "int",
          "value": 4
        }
      ],
      "name": "main"
    }
  ]
}
`;

  let datalogRules = `.decl var_live(line: symbol, var: symbol) .output
.decl var_def(line: symbol, var: symbol) .input
.decl var_use(line: symbol, var: symbol) .input
.decl next_line(line1: symbol, line2: symbol) .input

.rule var_live(L1, V) :- var_use(L1, V)
.rule var_live(L1, V) :- var_live(L2, V), next_line(L1, L2), !var_def(L1, V)`;

  let output = "";

  async function analyze() {
    try {
      // TODO: Replace with actual WASM call
      const result = await analyze_bril_program(brilCode);
      output = result;
    } catch (error) {
      output = `Error: ${error.message}`;
    }
  }
</script>

<main>
  <h1>Datalog Bril Dataflow analysis</h1>

  <div class="editors">
    <CodeEditor label="Bril Program" bind:value={brilCode} />
  </div>

  <button on:click={analyze}>Analyze Program</button>

  <div class="output">
    <h2>Analysis Output</h2>
    <pre>{output}</pre>
  </div>
</main>

<style>
  main {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }

  h1 {
    color: #f8f8f8;
    margin-bottom: 2rem;
  }

  .editors {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
    margin-bottom: 2rem;
  }

  button {
    background-color: #4caf50;
    color: white;
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    margin-bottom: 2rem;
  }

  button:hover {
    background-color: #45a049;
  }

  .output {
    background-color: #333;
    padding: 1rem;
    border-radius: 4px;
  }

  pre {
    margin: 0;
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>
