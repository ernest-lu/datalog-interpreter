
I've implemented datalog analysis for general datalog rules and facts. This can be used on the web-demo at https://ernest-lu.github.io/datalog-interpreter/. 

I've also implemented liveness analysis of facts generated from bril programs using the same datalog engine. 

Unfortunately, this does not yet work on the web-demo due to issues I had with importing the bril_rs and brilirs functions into the wasm. It seems like these have platform specific code in them that cannot be used when compiling to wasm.

We can still run this analysis in the CLI with a command like: 
```
cargo cargo run samples/dataflow/liveness/fibonacci.json | bril2txt 
```
Which should run datalog analysis on the fibonacci program and output the liveness optimized bril program code!

For a simple liveness sanity check, something like:
```
@main() {
  x: int = const 3;
  y: int = const 5;
  y: int = add x y;
  print y;
  x: int = const 4;
}
```

gets optimized to:
```
@main {
  x: int = const 3;
  y: int = const 5;
  y: int = add x y;
  print y;
}
```