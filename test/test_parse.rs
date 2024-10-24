use crate::parse::parse_program;

#[test]
fn test_parse_prorgram() {
    let program_input = ".decl edge(x, y) .input;\n.decl reachable(x, y) .output;\n.rule reachable(x, y) :- 1 edge(x, y);\n.rule reachable(x, z) :- 2 reachable(x, y), edge(y, z);";
    let program = parse_program(&program_input).unwrap();
    dbg!(&program);
}

#[test]
fn test_parse() {}
