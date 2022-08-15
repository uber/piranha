from tree_sitter import Language, Parser

Language.build_library(
  'build/my-languages.so',
  [
    'vendor/tree-sitter-query',
  ]
)

QUERY_LANGUAGE = Language('build/my-languages.so', 'query')
parser = Parser()
parser.set_language(QUERY_LANGUAGE)

def filter_eq(tag, value):
    return f'(#eq? @{tag} "{value}")'

def filter_not_eq(tag, value):
    return f'(#not-eq? @{tag} "{value}")'

def query(s_exp, constraints):
    template = """(
{}
{}
)"""
    constraints_str = "\n".join(constraints)
    return template.format(s_exp, constraints_str)


def rename_tags(query_str, renames):
    query_str_clone = query_str
    tree = parser.parse(bytes(query_str_clone, "utf8"))
    query = QUERY_LANGUAGE.query("((capture) @capture)")
    captures = query.captures(tree.root_node)
    tag_range = {}
    for c in captures:
        tag_range[c[0].text.decode('utf-8')] = (c[0].start_byte, c[0].end_byte)
    for k, v in renames.items():
        query_str_clone = query_str_clone[0:tag_range['@'+k][0]] + '@' + v +  query_str_clone[tag_range['@'+k][1]+1:]
    return query_str_clone


def transform_tags(query_str, rename_fn):
    query_str_clone = query_str
    tree = parser.parse(bytes(query_str_clone, "utf8"))
    query = QUERY_LANGUAGE.query("((capture) @capture)")
    captures = query.captures(tree.root_node)
    tag_range = {}
    for c in captures:
        tag_name = c[0].text.decode('utf-8')[1:]
        new_tag_name = rename_fn(tag_name)
        query_str_clone = query_str_clone[0:c[0].start_byte] + '@' + new_tag_name +  query_str_clone[c[0].end_byte + 1:]
    return query_str_clone

java_import_declaration_type = "((import_declaration (scoped_identifier (scoped_identifier)@type_qualifier (identifier)@type_name) @imported_type) @import)"
java_file = "((program) @compilation_unit)"

java_method_invocation = """((method_invocation 
	object: (_) @receiver
    name: (_) @name
    arguments : (_) @arguments
) @method_invocation)"""

java_type_identifier = "((type_identifier) @type_identifier)"


java_method_Declaration = "((method_declaration name : (_) @name)) @method_declaration"

