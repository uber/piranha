def filter_eq(tag, value):
    return '(#eq? @tag "value")'

def filter_not_eq(tag, value):
    return '(#not-eq? @tag "value")'

def query(s_exp, constraints):
    template = """(
{s_exp}
{constraints_str}
)"""
    constraints_str = "\n".join(constraints)
    return template.format(s_exp, constraints_str)


def rename_tags(query, renames):
    nq = query
    for k, v in renames.items():
        nq = nq.replace('@'+k, v)
    return nq

java_import_declaration_type = "((import_declaration (scoped_identifier (scoped_identifier)@type_qualifier (identifier)@type_name) @imported_type) @import)"
java_file = "((program) @compilation_unit)"

java_method_invocation = """((method_invocation 
	object: (_) @receiver
    name: (_) @name
    arguments : (_) @arguments
) @method_invocation)"""

java_type_identifier = "((type_identifier) @type_identifier)"


java_method_Declaration = "((method_declaration name : (_) @name)) @method_declaration"

