
I would like to implement a Datalog interpreter in Rust. This should be a simple rust program that takes in a .dl Datalog file along with a set of facts as a .in file and outputs the resulting datalog facts to stdout. 

Implementation-wise, the program should perform semi-naive evaluation of rules. The program loops through each rule, expanding rules by performing a hash join on the facts in the rule's body to output a new fact of the rule's head.

We will then construct a small program that converts .bril programs into a set of datalog facts about the dataflow of the program. We will use our datalog interpreter to expand these facts and then try to create a new datalog program off these expanded facts. Afterwords, we would like to create a web demo by compiling our programs into webassembly and running the bril program optimization in the browser.