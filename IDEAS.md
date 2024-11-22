- Datalog programs have been applied to data integration, networking, and program analysis
- Implement semi-naive evaluation
    - Ask for explanation on this. (Expanding the frontier)?
- Cross-language function calls on datalog
- Datalog + static analysis
    - Find security vulnerabilities in code
    - Run static analysis on bril programs

- Why embed languages
 - Programmers like to use libraries to express their language
 - SQLLite is an embedded database engine
 
 - Lambdas
 - Recursive closures
 - Datalog static analysis of rust code
 - People like using datalog to figure out mutually recursive information about their programs
 - Datalog for debugging
 - Sanitizer implemented
 - Datalog for idea generation
 - Datalog for input generation
    - Variables and bounds
- Datalog to search state spaces?

- Working on pytorch
- Pytorch relations to compiler optimizations


- Buildit implements multi-stage compiling

- Plan for liveness analysis
    - Implement a node at each instruction point
        - Includes which variables are live at each instruction point
        - Includes successor relations (input)
        - Use var and definitions all here
        - Naming scheme: 
            - Independently handle functions
                - block_name.line_name

- Understanding of negation
    - Needs to be stratified negation
        - Meaning negation cannot be in heads
        - Negation cannot form cycles
    -  Don't necessarily need to implement negation for this datalog

- Understanding of join optimization
    - Want to order joins in such a way that keys line up with each other
    - If two tables share a key, it is better to join them together first. Want to avoid implementing a cartesian product

- Understanding of hydroflow:
    - Dataflow analysis framework for general data processing.
    - Not necesserily a compiler. Compiler dataflow runs on the control flow graph as opposed to data flow graph.
