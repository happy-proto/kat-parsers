use tree_sitter::Parser;
use tree_sitter_kat_csh::LANGUAGE;

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

fn count_kind(node: tree_sitter::Node<'_>, kind: &str) -> usize {
    let mut cursor = node.walk();
    usize::from(node.kind() == kind)
        + node
            .children(&mut cursor)
            .map(|child| count_kind(child, kind))
            .sum::<usize>()
}

#[test]
fn virtual_environment_activation_and_substitutions() {
    let source = r#"#!/bin/tcsh
set newline='\
'
alias deactivate 'test $?_OLD_PATH != 0 && setenv PATH "$_OLD_PATH:q"; test "\!:*" != "keep" && unalias deactivate'
deactivate keep
setenv VIRTUAL_ENV '/opt/example/.venv'
set _OLD_PATH="$PATH:q"
setenv PATH "$VIRTUAL_ENV:q/bin:$PATH:q"
if ('demo' != "") then
    setenv VIRTUAL_ENV_PROMPT 'demo'
else
    setenv VIRTUAL_ENV_PROMPT "$VIRTUAL_ENV:t:q"
endif
if ( $?prompt ) then
    set _OLD_PROMPT="$prompt:q"
    if ( "$prompt:q" =~ *"$newline:q"* ) then
        :
    else
        set prompt = '('"$VIRTUAL_ENV_PROMPT:q"') '"$prompt:q"
    endif
endif
unset newline
alias pydoc python -m pydoc
rehash
"#;
    let tree = parse(source);
    assert_eq!(count_kind(tree.root_node(), "if_statement"), 3);
    assert_eq!(count_kind(tree.root_node(), "else_clause"), 2);
    for variable in ["$?prompt", "$PATH:q", "$VIRTUAL_ENV:t:q"] {
        let offset = source.find(variable).unwrap();
        let node = tree
            .root_node()
            .descendant_for_byte_range(offset, offset + variable.len())
            .unwrap();
        assert_eq!(node.kind(), "variable", "{variable}");
        assert_eq!(node.byte_range(), offset..offset + variable.len());
    }
}

#[test]
fn control_flow_pipelines_arrays_and_word_punctuation() {
    let tree = parse(
        r#"set files = (src/*.c /opt/a-b/a_b:c.txt)
foreach file ($files)
    if (-f "$file") echo "$file" >>&! build.log
end
@ count = 0
while ($count < 3 && $?HOME)
    @ count++
    echo $count |& tee -a build.log
end
switch ($argv[1])
case build:
    echo "building ${HOME}/src"; breaksw
case *:
    echo `basename "$cwd"` && echo done
    breaksw
default:
    exit 1
endsw
if ($status == 0) then
    echo ok
else if ($status == 1) then
    echo retry
else
    echo failed
endif
echo $#argv $argv[*]:q $$ $! $0 $< \
    "dollar $ and \"quote\"" 'literal # $HOME'
"#,
    );
    for (kind, count) in [
        ("foreach_statement", 1),
        ("while_statement", 1),
        ("switch_statement", 1),
        ("case_clause", 3),
        ("if_statement", 2),
        ("else_if_clause", 1),
        ("command_substitution", 1),
    ] {
        assert_eq!(
            count_kind(tree.root_node(), kind),
            count,
            "{kind}: {}",
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn incomplete_input_keeps_earlier_command() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse("setenv NAME example\nif (", None).unwrap();
    assert_eq!(
        tree.root_node().named_child(0).unwrap().kind(),
        "command_list"
    );
}

#[test]
fn control_words_as_command_arguments_do_not_end_blocks() {
    let tree = parse("if (1) then\n  echo endif end else\nendif\n");
    assert_eq!(count_kind(tree.root_node(), "if_statement"), 1);
    let mut words = Vec::new();
    fn visit(node: tree_sitter::Node<'_>, source: &str, words: &mut Vec<String>) {
        if node.kind() == "word" {
            words.push(source[node.byte_range()].to_owned());
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(child, source, words);
        }
    }
    visit(
        tree.root_node(),
        "if (1) then\n  echo endif end else\nendif\n",
        &mut words,
    );
    assert_eq!(words, ["echo", "endif", "end", "else"]);
}
