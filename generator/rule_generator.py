import glob
import os
import random
import re
import string

import toml
import yaml


def parse_rules_templates(file_path):
    with open(file_path, 'r') as file:
        data = toml.load(file)

    templates_map = {}

    if 'template' in data and isinstance(data['template'], list):
        for template in data['template']:
            if 'name' in template and 'query' in template:
                if 'type' in template:
                    template_type = template['type']
                    templates_map[template['name'] + '__' + template_type] = template['query']
                else:
                    templates_map[template['name']] = template['query']

    return templates_map


def parse_rules(file_path):
    with open(file_path, 'r') as file:
        data = toml.load(file)

    rules_map = {}

    if 'rules' in data and isinstance(data['rules'], list):
        for rule in data['rules']:
            rules_map[rule['name']] = rule

    return rules_map


def extract_closing_tags(text):
    tags = set()
    pos = 0

    while True:
        start_close_tag = text.find('%{/', pos)
        if start_close_tag == -1:
            break
        end_close_tag = text.find('}%', start_close_tag)
        if end_close_tag == -1:
            break

        tag_name = text[start_close_tag + 3:end_close_tag]
        tags.add(tag_name)

        pos = end_close_tag + 2
    return list(tags)


def apply_template_with_blocks(text, template_type):
    map = {}

    def replace_block(match):
        template_name = match.group(1)
        inner_content = match.group(2).strip()

        tags = extract_closing_tags(inner_content)
        for tag in tags:
            if tag in map:
                map[tag] += 1
            else:
                map[tag] = 1

        key = template_name if template_type is None else template_name + '__' + template_type
        if key not in rule_templates_map:
            key = template_name

        if key in rule_templates_map:
            template = rule_templates_map[key].strip()
            if template_name in map:
                template = re.sub(r'@@out_\w+@@', '', template)

            return template.replace("%{content}%", inner_content)
        return match.group(0)

    pattern = r'%\{(\w+)\}%((?:[^%]|%(?!{/?_?\1\}))*?)%{/\1}%'

    modified = True
    while modified:
        text, count = re.subn(pattern, replace_block, text, flags=re.DOTALL)
        modified = count > 0

    return text


def load_rules_graph(yml_path):
    with open(yml_path, 'r') as file:
        graph_data = yaml.safe_load(file)['graph']

    nodes = graph_data.get('nodes', {})
    start_nodes = [('Global', node, None) for node in graph_data.get('start')]

    def build_graph(current_node, visited):
        if visited.get(current_node, 0) >= 1:
            return {}

        visited[current_node] = visited.get(current_node, 0) + 1

        graph = {}
        edges = []

        node_name = current_node[1]
        node_type = None
        target_node = None
        for node in nodes:
            if '__' in node:
                name, node_type = node.split('__')
            else:
                name = node
            if name == node_name:
                target_node = nodes[node]
                break
        if target_node is None:
            return None

        for scope in target_node:
            for edge in target_node[scope]:
                edges.append((scope, edge, node_type))

        for edge in edges:
            subgraph = build_graph(edge, visited.copy())
            graph[edge] = subgraph

        return graph if graph else None

    graphs = []

    for start_node in start_nodes:
        graph = {start_node: {}}
        parts = start_node[1].split('__')
        key = parts[0]
        key_scopes = [None]
        if 'expression' not in key:
            key_scopes.append('cleanup')
        edges = []
        if nodes is not None:
            for key_scope in key_scopes:
                k = key + ('' if key_scope is None else ('__' + key_scope))
                if k in nodes:
                    for scope in nodes[k]:
                        for item in nodes[k][scope]:
                            edges.append((scope, item, key_scope))
            if len(edges) > 0:
                for edge in edges:
                    subgraph = build_graph(edge, {})
                    graph[start_node][edge] = subgraph  # Add subgraph regardless of its value (including None)
        graphs.append(graph)

    return graphs, nodes


def generate_random_string(length):
    first_char = random.choice(string.ascii_letters)
    remaining_chars = string.ascii_letters + string.digits
    random_string = ''.join(random.choice(remaining_chars) for i in range(length - 1))
    return (first_char + random_string).lower()


def generate_query(src_query, template_type, prev_ctx):
    def get_query_context(src_query):
        def generate_random_postfix(length=4):
            return ''.join(random.choices(string.ascii_lowercase + string.digits, k=length))

        ctx = {}
        for tag in ['in', 'out']:
            ctx[tag] = {}
            pattern = fr'@@{tag}_(\w+)@@'
            placeholders = re.findall(pattern, src_query)
            for placeholder in placeholders:
                var_name = f"@{placeholder}_{generate_random_postfix()}"
                ctx[tag][placeholder] = var_name
        return ctx

    src_query = apply_template_with_blocks(src_query, template_type)

    while '%{' in src_query:
        for key, value in rule_templates_map.items():
            placeholder = '%{' + key + '}%'
            if placeholder in src_query:
                if template_type is not None:
                    value = rule_templates_map[key + '__' + template_type]
                else:
                    value = rule_templates_map[key]
            src_query = src_query.replace(placeholder, value)

    curr_ctx = get_query_context(src_query)

    for in_var in curr_ctx['in']:
        src_query = src_query.replace(f'@@in_{in_var}@@', '@' + in_var)

    for out_var in curr_ctx['out']:
        src_query = src_query.replace(f'@@out_{out_var}@@', curr_ctx['out'][out_var])

    if template_type == 'literal':
        src_query = f"{src_query}\n(#eq? @flag_name \"\\\\\\\"@stale_flag_name\\\\\\\"\")\n"
    else:
        for in_var in curr_ctx['in']:
            if in_var not in prev_ctx['out'] \
                    and in_var in curr_ctx['out'] \
                    and 'flag_alias' not in in_var:  # dirty hack
                src_query = src_query.replace(curr_ctx['out'][in_var], '')
                src_query = src_query.replace('@' + in_var, '')
                del curr_ctx['out'][in_var]
                continue

            replacement = prev_ctx["out"][in_var] if in_var in prev_ctx["out"] else "@stale_flag_name"
            pattern = re.compile(rf'(\(#(?:match|eq)\?\s@body\s)(@{in_var}|"@{in_var}")(\))')

            def replace_placeholder(match):
                prefix, placeholder, suffix = match.groups()
                if placeholder.startswith('"') and placeholder.endswith('"'):
                    return f'{prefix}"{replacement}"{suffix}'
                else:
                    return f"{prefix}{replacement}{suffix}"

            output_string, replacement_count = pattern.subn(replace_placeholder, src_query)
            if replacement_count == 0:
                src_query = f'{src_query}\n(#eq? @{in_var} "{replacement}")\n'
            else:
                src_query = output_string

    return f'\n(\n {src_query}\n )\n', curr_ctx


def escape_string(value):
    """
    Escapes special characters in a string for TOML format.
    """
    return value.replace('\\', '\\\\').replace('\n', '\\n').replace('\t', '    ').replace('"', '\\"')


def format_value(value):
    """
    Formats a value for TOML output.
    """
    if isinstance(value, str):
        if '\n' in value:
            return '"""' + value.replace('"""', '\\"""') + '"""'
        else:
            return '"' + escape_string(value) + '"'
    elif isinstance(value, bool):
        return 'true' if value else 'false'
    elif value is None:
        return 'null'
    else:
        return str(value)


cache = dict()
cache['edges'] = dict()
cache['rules'] = dict()


def write_dict_to_toml(data, key, base_file_path, file_name):
    def write_table(table, indent=''):
        lines = []
        for k, v in table.items():
            if isinstance(v, list):
                if all(isinstance(i, dict) for i in v):
                    for item in v:
                        cache_key = None
                        if k == 'edges':
                            cache_key = item['from'] + item['scope'] + ','.join(item['to'])
                        elif k == 'rules':
                            cache_key = item['name']
                        if cache_key is not None:
                            if cache_key in cache[k]:
                                continue
                            else:
                                cache[k][cache_key] = True
                        lines.append('\n')
                        lines.append(f"{indent}[[{k}]]")
                        lines.extend(write_table(item, indent + ''))
                else:
                    lines.append(f"{indent}{k} = [{', '.join(format_value(i) for i in v)}]")
            elif isinstance(v, dict):
                lines.append(f"{indent}[{k}]")
                lines.extend(write_table(v, indent + '    '))
            else:
                lines.append(f"{indent}{k} = {format_value(v)}")
        return lines

    toml_lines = write_table({key: data[key]})

    subdirectories = [d for d in glob.glob(os.path.join(base_file_path, '*/')) if os.path.isdir(d)]

    for subdir in subdirectories:
        config_dir = os.path.join(subdir, 'control/configurations')
        file_path = os.path.join(config_dir, file_name)
        with open(file_path, 'a') as file:
            file.write('\n'.join(toml_lines))
            file.write('\n')


def clean_files(base_file_path, file_name):
    """
    Cleans the contents of the specified file.

    Args:
        file_path (str): The path to the file to clean.
    """
    subdirectories = [d for d in glob.glob(os.path.join(base_file_path, '*/')) if os.path.isdir(d)]

    for subdir in subdirectories:
        config_dir = os.path.join(subdir, 'control/configurations')
        file_path = os.path.join(config_dir, file_name)
        with open(file_path, 'w') as file:
            pass


def all_children_cleanup(graph):
    return graph is not None and len(graph) > 0 and all(c[2] == 'cleanup' for c in graph)


def write_paths_to_toml(graphs, nodes):
    # todo: is a copy-paste from traverse_and_write

    def traverse_cleanup(graph, current_path, prefix, parent_ctx):
        if graph is None:
            edges = {"edges": []}
            rules = {"rules": []}
            var_ctx = {'in': {}, 'out': {}}
            for i in range(len(current_path)):
                rule_name, template_type = (current_path[i][1].split('__', 1) + [None])[:2]
                rule = rules_map[rule_name]

                ctx_copy = var_ctx.copy()
                ctx_copy['out'] = parent_ctx['out'].copy()
                rule_query, next_ctx = generate_query(rule['query'], template_type, ctx_copy)

                if i < len(current_path) - 1:
                    edges["edges"].append({
                        'scope': current_path[i + 1][0].capitalize(),
                        'from': prefix + '_' + str(i) + '_' + current_path[i][1],
                        'to': [prefix + '_' + str(i + 1) + '_' + current_path[i + 1][1]]
                    })

                holes = ['treated', 'untreated', 'stale_flag_name'] + rule.get('holes', [])
                for out_var in var_ctx['out']:
                    holes.append(var_ctx['out'][out_var].replace('@', ''))

                new_rule = {
                    'name': prefix + '_' + str(i) + '_' + current_path[i][1],
                    'query': rule_query,
                    'holes': holes,
                    'is_seed_rule': False
                }

                if 'groups' in rule:
                    new_rule['groups'] = rule['groups']

                if 'replace_node' in rule:
                    new_rule['replace_node'] = rule['replace_node']
                    if 'replace' in rule:
                        new_rule['replace'] = rule['replace']
                    else:
                        new_rule['replace'] = ''

                rules["rules"].append(new_rule)
                var_ctx = next_ctx

            write_dict_to_toml(edges, 'edges', '../test-resources/kt/feature_flag_system_tests', 'edges.toml')
            write_dict_to_toml(rules, 'rules', '../test-resources/kt/feature_flag_system_tests', 'rules.toml')
        else:
            for node, subgraph in graph.items():
                traverse_cleanup(subgraph, current_path + [node], prefix, parent_ctx)

    def traverse_and_write(graph, current_path, prefix=None):
        def grouped_by_scope(subgraph):
            if subgraph is None:
                return [None]
            grouped = {}
            for key, value in subgraph.items():
                scope = key[0]
                if scope not in grouped:
                    grouped[scope] = {}
                grouped[scope][key] = value

            return [group for group in grouped.values()]

        if graph is None or all_children_cleanup(graph):
            edges = {"edges": [
                {
                    'scope': 'Parent',
                    'from': 'delete_all_statements_after_return',
                    'to': ['delete_all_statements_after_return']
                }
            ]}
            rules = {"rules": []}
            var_ctx = {'out': {}, 'in': {}}
            prev_ctx = None

            last_rule_key = current_path[-1][1].split('__')[0]
            last_rule = rules_map[last_rule_key]
            if 'terminal' in last_rule and not last_rule['terminal']:
                return None

            for i in range(len(current_path)):
                rule_name, template_type = (current_path[i][1].split('__', 1) + [None])[:2]
                rule = rules_map[rule_name]
                rule_query, next_ctx = generate_query(rule['query'], template_type, var_ctx)

                if i < len(current_path) - 1:
                    edges["edges"].append({
                        'scope': current_path[i + 1][0].capitalize(),
                        'from': prefix + '_' + str(i) + '_' + current_path[i][1],
                        'to': [prefix + '_' + str(i + 1) + '_' + current_path[i + 1][1]]
                    })
                elif len(current_path) == 1:
                    edges["edges"].append({
                        'scope': current_path[i][0].capitalize(),
                        'from': prefix + '_' + current_path[i][1],
                        'to': [prefix + '_' + current_path[i][1]]
                    })

                holes = ['treated', 'untreated', 'stale_flag_name'] + rule.get('holes', [])
                for out_var in var_ctx['out']:
                    holes.append(var_ctx['out'][out_var].replace('@', ''))

                new_rule = {
                    'name': prefix + '_' + str(i) + '_' + current_path[i][1],
                    'query': rule_query,
                    'holes': holes
                }
                if i > 0:
                    new_rule['is_seed_rule'] = False

                if 'groups' in rule:
                    new_rule['groups'] = rule['groups']

                if i == len(current_path) - 1:
                    if graph is not None:
                        edges["edges"].append({
                            'scope': next(iter(graph.items()))[0][0].capitalize(),
                            'from': prefix + '_' + str(i) + '_' + current_path[i][1],
                            'to': [prefix + '_0_' + c[1] for c in graph]
                        })
                    if 'replace_node' in rule:
                        new_rule['replace_node'] = rule['replace_node']
                        if 'replace' in rule:
                            new_rule['replace'] = rule['replace']
                        else:
                            new_rule['replace'] = ''

                if 'filters' in rule:
                    new_rule['rules.filters'] = rule['filters']
                rules["rules"].append(new_rule)
                prev_ctx = var_ctx
                var_ctx = next_ctx

            write_dict_to_toml(edges, 'edges', '../test-resources/kt/feature_flag_system_tests', 'edges.toml')
            write_dict_to_toml(rules, 'rules', '../test-resources/kt/feature_flag_system_tests', 'rules.toml')

            return prev_ctx

        elif graph is not None:
            for node, subgraph in graph.items():
                prefix = generate_random_string(5)
                for sg in grouped_by_scope(subgraph):
                    parent_ctx = traverse_and_write(
                        sg, current_path + [node], prefix)
                    if all_children_cleanup(sg):
                        for _n, _g in sg.items():
                            traverse_cleanup(_g, [_n], prefix, parent_ctx)

            prefix = generate_random_string(5)
            traverse_and_write(
                None, current_path, prefix)

    for idx, graph in enumerate(graphs):
        for start_node, edges in graph.items():
            traverse_and_write(edges, [start_node])


file_path = 'rule_templates.toml'
rules_graph_path = 'edges.yml'
input_file_path = 'rules.toml'

clean_files('../test-resources/kt/feature_flag_system_tests', 'edges.toml')
clean_files('../test-resources/kt/feature_flag_system_tests', 'rules.toml')

rule_templates_map = parse_rules_templates(file_path)

rules_map = parse_rules(input_file_path)

rules_graph, nodes = load_rules_graph(rules_graph_path)
write_paths_to_toml(rules_graph, nodes)
