# DatalogInt

A small datalog interpreter written in Rust. Can interpret .dl programs and run them on a set of input facts. Implemented by a naive expansion of rules until convergence.


## Example


Datalog program:
```datalog
.decl edge(x, y) .input;
.decl reachable(x, y) .output;
.rule reachable(x, y) :- 1 edge(x, y);
.rule reachable(x, z) :- 2 reachable(x, y), edge(y, z);
```

Input: 
```input
2
edge(x, y);
edge(y, z);
```

Output:

```
reachable(x, y);
reachable(x, z);
reachable(y, z);
```
