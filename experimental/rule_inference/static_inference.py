from tree_sitter import Node, TreeCursor
from typing import List


class QueryWriter:
    capture_groups = {}
    count = 0
    query_str = ""

    def write(self, seq_nodes: List[Node]):
        # Get textual representation of the sequence
        # Then find for each named child of source_node, can we replace it with its respective target group

        node_queries = []
        for node in seq_nodes:
            node_queries.append(self.write_query(node))

        self.query_str = ".".join(node_queries)

        return self.query_str

    def write_query(self, node: Node, depth=0, prefix=""):
        indent = " " * depth
        cursor: TreeCursor = node.walk()
        s_exp = indent + f"{prefix}({node.type} "

        next_child = cursor.goto_first_child()
        while next_child:
            child_node: Node = cursor.node
            if child_node.is_named:
                s_exp += "\n"
                prefix = ""
                if cursor.current_field_name():
                    prefix = f"{cursor.current_field_name()}: "
                s_exp += self.write_query(cursor.node, depth + 1, prefix)
            next_child = cursor.goto_next_sibling()

        self.count += 1
        self.capture_groups[f"@tag{self.count}"] = node
        return s_exp + f") @tag{self.count}"
