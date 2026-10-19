use tree_sitter::Parser;
use tree_sitter_kat_jam::LANGUAGE;

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    assert!(
        !tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
    tree
}

#[test]
fn parses_b2_build_language() {
    let tree = parse(
        r#"# Build configuration
import feature : feature ;
ROOT ?= /opt/sdk ;
local sources = main.cpp helper.cpp ;
rule linking ( properties * )
{
    local result ;
    if <target-os>windows in $(properties) && ! <link>static in $(properties)
    { result += <library>ws2_32 ; }
    else if ( $(ROOT) != "" ) { result += <include>$(ROOT)/include ; }
    switch [ os.name ] { case linux : result += <library>dl ; case * : ECHO other ; }
    return $(result) ;
}
for local source in $(sources) { ECHO "building $(source)" ; }
while $(pending) { pending = [ next $(pending) ] ; }
module demo { local rule helper ( args * ) { return $(args[1]:J=,) ; } }
class target : base { rule __init__ ( name ) { self.name = $(name) ; } }
FLAGS on app = -O2 ;
on app { ECHO [ on app return $(FLAGS) ] ; }
include settings.jam ;
actions quietly together compile bind FLAGS
{
    echo "$(<)" ; if true; then echo { nested }; fi
}
exe app : $(sources) : <link>shared <include>$(ROOT)/include ;
"#,
    );
    let sexp = tree.root_node().to_sexp();
    for role in [
        "rule_definition",
        "rule_call",
        "variable_expansion",
        "if_statement",
        "actions_definition",
        "case_statement",
    ] {
        assert!(sexp.contains(role), "missing {role}");
    }
}

#[test]
fn preserves_word_boundaries_and_nested_expansions() {
    let tree = parse(
        "X = pre$(name)suffix \"space $(x)\" $($(names[1]):J=,) ;\n#| block\ncomment |#\nECHO 中文 ;\n",
    );
    let assignment = tree.root_node().named_child(0).unwrap();
    assert_eq!(
        assignment.named_child_count(),
        4,
        "{}",
        assignment.to_sexp()
    );
}

#[test]
fn parses_embedded_properties_file_generation_and_regex_dollars() {
    parse(
        r#"local flags = <variant>release:<define>NDEBUG <linkflags>"/OPT:REF" ;
local pattern = "^lib(.*)\\.a$" ;
local config = @("sample.pc":E=$(flags)) ;
install (bindir)/sample : sample : <location>(libdir) ;
"#,
    );
}

#[test]
fn retains_complete_statements_before_unfinished_input() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser
        .parse("import feature ;\nrule unfinished (", None)
        .unwrap();
    assert_eq!(
        tree.root_node().named_child(0).unwrap().kind(),
        "call_statement"
    );
}
