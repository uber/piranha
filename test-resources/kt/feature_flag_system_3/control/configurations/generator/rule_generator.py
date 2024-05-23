import random
import re
import string

import toml
import yaml


def parse_rules_templates(file_path):
    # Load the TOML file
    with open(file_path, 'r') as file:
        data = toml.load(file)

    # Initialize a dictionary to store the templates
    templates_map = {}

    # Check if the 'template' key exists and is a list
    if 'template' in data and isinstance(data['template'], list):
        # Iterate through each template in the list
        for template in data['template']:
            # Check if both 'name' and 'query' keys exist in the template
            if 'name' in template and 'query' in template:
                # Add the 'name' and 'query' to the dictionary
                if 'type' in template:
                    template_type = template['type']
                    templates_map[template['name'] + '__' + template_type] = template['query']
                else:
                    templates_map[template['name']] = template['query']

    return templates_map


def parse_rules(file_path):
    # Load the TOML file
    with open(file_path, 'r') as file:
        data = toml.load(file)

    # Initialize a dictionary to store the templates
    rules_map = {}

    # Check if the 'template' key exists and is a list
    if 'rules' in data and isinstance(data['rules'], list):
        # Iterate through each template in the list
        for rule in data['rules']:
            rules_map[rule['name']] = rule

    return rules_map


def extract_closing_tags(text):
    tags = set()
    pos = 0

    while True:
        # Find the closing tag pattern
        start_close_tag = text.find('%{/', pos)
        if start_close_tag == -1:
            break
        end_close_tag = text.find('}%', start_close_tag)
        if end_close_tag == -1:
            break

        # Extract the tag name
        tag_name = text[start_close_tag + 3:end_close_tag]

        # Add the tag name to the set
        tags.add(tag_name)

        # Move the position to search for the next tag
        pos = end_close_tag + 2

    return list(tags)


def apply_template_with_blocks(text, template_type):
    # Function to replace each found block with the template and inner content

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
                template = template.replace('@@out@@', '')
            # Insert the inner content into the template
            # Assuming that there's a placeholder like "{content}" in the template where inner content should go
            return template.replace("%{content}%", inner_content)
        return match.group(0)  # Return the original text if no template is found

    # Initial pass
    pattern = r'%\{(\w+)\}%((?:[^%]|%(?!{/?_?\1\}))*?)%{/\1}%'

    # Apply replacement iteratively until no more changes
    modified = True
    while modified:
        text, count = re.subn(pattern, replace_block, text, flags=re.DOTALL)
        modified = count > 0

    return text

def load_rules_graph(yml_path):
    with open(yml_path, 'r') as file:
        graph_data = yaml.safe_load(file)['graph']

    # Extract sections from the YAML structure
    start_nodes = graph_data.get('start', {})
    nodes = graph_data.get('nodes', {})

    # Function to recursively build the graph
    def build_graph(current_node, visited):
        # if current_node in terminal_nodes:
        #     return None  # Return None for terminal nodes

        # Increase visit count and check if visited more than twice
        if visited.get(current_node, 0) >= 2:
            return {}  # Skip if visited more than twice

        # Update the visited count for this node
        visited[current_node] = visited.get(current_node, 0) + 1

        graph = {}
        # Get the edges for the current node from the nodes section
        edges = nodes.get(current_node, [])
        for edge in edges:
            subgraph = build_graph(edge, visited.copy())  # Use a copy to not share state between branches
            graph[edge] = subgraph  # Add subgraph regardless of its value (including None)

        return graph if graph else None  # Return None if no further edges

    # Initialize the list of graphs
    graphs = []

    # Build a graph for each starting node
    for start_node, _ in start_nodes.items():
        graph = {start_node: {}}
        key = start_node.split('__')[0]
        edges = None
        if key in nodes:
            edges = nodes[key]
        if edges is not None:
            for edge in edges:
                subgraph = build_graph(edge, {})
                graph[start_node][edge] = subgraph  # Add subgraph regardless of its value (including None)
        graphs.append(graph)

    return graphs


def generate_random_string(length):
    first_char = random.choice(string.ascii_letters)
    remaining_chars = string.ascii_letters + string.digits
    random_string = ''.join(random.choice(remaining_chars) for i in range(length - 1))
    return (first_char + random_string).lower()


def generate_query(src_query, template_type, in_var, out_var):
    src_query = apply_template_with_blocks(src_query, template_type)

    while '%{' in src_query:
        for key, value in rule_templates_map.items():
            # Construct the placeholder pattern
            placeholder = '%{' + key + '}%'
            # Replace the placeholder in the content
            if placeholder in src_query:
                if template_type is not None:
                    value = rule_templates_map[key + '__' + template_type]
                else:
                    value = rule_templates_map[key]
            src_query = src_query.replace(placeholder, value)

    if in_var:
        src_query = src_query.replace('@@in@@', '@flag_name')
    else:
        src_query = src_query.replace('@@in@@', f'@{out_var}')
    src_query = src_query.replace('@@out@@', '@' + out_var)

    if template_type == 'literal':
        src_query = f"{src_query}\n(#eq? @flag_name \"\\\\\\\"@stale_flag_name\\\\\\\"\")\n"
    elif in_var is None:
        src_query = f'{src_query}\n(#eq? @{out_var} "@stale_flag_name")\n'

    else:
        src_query = f'{src_query}\n(#eq? @flag_name "@{in_var}")\n'

    return f'\n(\n {src_query}\n )\n'


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


def write_dict_to_toml(data, key, file_path):
    """
    Writes a dictionary to a TOML file, preserving multiline strings and converting tab characters to spaces.

    Args:
        data (dict): The dictionary to write to the TOML file.
        key (str): The top-level key for the dictionary.
        file_path (str): The path to the TOML file to write.
    """

    def write_table(table, indent=''):
        lines = []
        for k, v in table.items():
            if isinstance(v, list):
                if all(isinstance(i, dict) for i in v):
                    for item in v:
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
    with open(file_path, 'a') as file:
        file.write('\n'.join(toml_lines))


def clean_file(file_path):
    """
    Cleans the contents of the specified file.

    Args:
        file_path (str): The path to the file to clean.
    """

    with open(file_path, 'w') as file:
        pass  # Opening in write mode with 'w' will truncate the file


def write_paths_to_toml(graphs):
    def traverse_and_write(graph, current_path, prefix=None):
        # Check if graph is a leaf node (string and not a dict)
        if graph is None:
            # Create TOML content for the current path
            edges = {"edges": []}
            rules = {"rules": []}
            in_var = None
            for i in range(len(current_path)):
                out_var = f'{generate_random_string(4)}_flag_alias'
                rule_name, template_type = (current_path[i].split('__', 1) + [None])[:2]
                rule = rules_map[rule_name]
                rule_query = generate_query(rule['query'], template_type, in_var, out_var)
                if i < len(current_path) - 1:
                    edges["edges"].append({
                        'scope': 'Global',
                        'from': prefix + '_' + current_path[i],
                        'to': [prefix + '_' + current_path[i + 1]]
                    })
                elif len(current_path) == 1:
                    edges["edges"].append({
                        'scope': 'Global',
                        'from': prefix + '_' + current_path[i],
                        'to': [prefix + '_' + current_path[i]]
                    })

                new_rule = {
                    'name': prefix + '_' + current_path[i],
                    'query': rule_query,
                    'holes': [var for var in [in_var, 'treated', 'untreated', 'stale_flag_name'] if var is not None]
                }
                if i > 0:
                    new_rule['is_seed_rule'] = False

                if 'groups' in rule:
                    new_rule['groups'] = rule['groups']

                if i == len(current_path) - 1:
                    new_rule['replace_node'] = rule['replace_node']
                    if 'replace' in rule:
                        new_rule['replace'] = rule['replace']
                    else:
                        new_rule['replace'] = ''

                rules["rules"].append(new_rule)
                in_var = out_var

            # Define file path
            write_dict_to_toml(edges, 'edges', f'../edges.toml')
            write_dict_to_toml(rules, 'rules', f'../rules.toml')

        elif isinstance(graph, dict):  # Ensure it's a dictionary before iterating
            for node, subgraph in graph.items():
                # Continue the path with the current node
                traverse_and_write(
                    subgraph, current_path + [node], generate_random_string(5))
            # generate rules for the current node
            traverse_and_write(
                None, current_path, generate_random_string(5))

    # Process each graph
    for idx, graph in enumerate(graphs):
        for start_node, edges in graph.items():
            # Start a new path from each start node
            traverse_and_write(edges, [start_node])


# Path to your TOML file
file_path = 'templates.toml'
# Path to your YML file
rules_graph_path = 'edges_template.yml'
# Path to the input TOML file you want to modify
input_file_path = 'rules_template.toml'

clean_file('../edges.toml')
clean_file('../rules.toml')

# Parse the file and print the results
rule_templates_map = parse_rules_templates(file_path)

# Parse the file and print the results
rules_map = parse_rules(input_file_path)

rules_graph = load_rules_graph(rules_graph_path)
write_paths_to_toml(rules_graph)
