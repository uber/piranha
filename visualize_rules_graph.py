@attr.s
class QueryWriter:
    """
    Class to represent a query writer for nodes.

    :ivar seq_nodes: Sequence of nodes for which the query will be written.
    :ivar template_holes: A dictionary to keep track of the nodes that will be replaced by a capture group.
    :ivar capture_groups: A dictionary to keep track of the nodes that will be captured in the query.
    :ivar count: A counter for naming the capture groups.
    :ivar query_str: The Comby pattern that represents the query.
    :ivar query_ctrs: List of constraints for the Comby pattern.
    :ivar outer_most_node: Represents the node that is currently the furthest from the root.
    """

    seq_nodes = attr.ib(type=list)
    template_holes = attr.ib(type=List)
    capture_groups = attr.ib(default=attr.Factory(dict))

    count = attr.ib(default=0)
    query_str = attr.ib(default="")
    query_ctrs = attr.ib(default=attr.Factory(list))
    outer_most_node = attr.ib(default=None)

    def write(self, simplify=False):
        """
        Get textual representation of the sequence.
        Find for each named child of source_node, can we replace it with its respective target group.

        :param simplify: If True, simplify the query.
        :return: The query string
        """

        self.query_str = ""
        self.query_ctrs = []

        node_queries = [
            self.write_query(node, simplify=simplify) for node in self.seq_nodes
        ]

        self.query_str = ".".join(node_queries) + "\n" + "\n".join(self.query_ctrs)
        self.query_str = f"({self.query_str})"

        return self.query_str

    def write_query(self, node: Node, depth=0, prefix="", simplify=False):
        """
        Write a query for a given node, considering its depth and prefix.

        :param node: The node for which the query will be written.
        :param depth: The current depth of the node.
        :param prefix: Prefix for the current node.
        :param simplify: If True, simplify the query.
        :return: The query string for this node.
        """

        indent = " " * depth
        cursor: TreeCursor = node.walk()

        self.count += 1
        node_repr = node.text.decode("utf8")
        node_name = f"@tag{self.count}n"

        if node_repr in self.template_holes:
            # Fixed node
            node_types = self.template_holes[node_repr]
            if len(node_types) == 1:
                s_exp = indent + f"{prefix}({node_types[0]})"
            else:
                alternations = " ".join([f"({node_type})" for node_type in node_types])
                s_exp = indent + f"{prefix}[{alternations}]"

        else:
            # Regular node
            node_type = node.type
            s_exp = indent + f"{prefix}({node_type} "
            next_child = cursor.goto_first_child()
            visited = 0
            while next_child:
                if cursor.node.is_named:
                    visited += 1
                    s_exp += self.strategy.process_child(cursor, depth)
                next_child = cursor.goto_next_sibling()

            # if the node is an identifier, add it to eq constraints
            if visited == 0:
                self.query_ctrs.append(
                    f"(#eq? {node_name} \"{node.text.decode('utf8')}\")"
                )
            s_exp += f")"

        self.capture_groups[node_name] = node
        self.outer_most_node = node_name
        return s_exp + f" {node_name}"

    def process_child(self, cursor: TreeCursor, depth: int) -> str:
        """Write the query as if the child node was the root of the tree."""
        prefix = (
            f"{cursor.current_field_name()}: " if cursor.current_field_name() else ""
        )
        return "\n" + self.write_query(cursor.node, depth + 1, prefix, simplify=False)
