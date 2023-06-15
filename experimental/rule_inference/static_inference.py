import attr
from tree_sitter import Node, TreeCursor
from typing import List, Dict
from patch import Patch
from node_utils import NodeUtils
import re


@attr.s
class QueryWriter:
    """
    This class writes a query for a given node considering its depth and prefix.
    """

    capture_groups = attr.ib(factory=dict)
    count = attr.ib(default=0)
    query_str = attr.ib(default="")
    query_ctrs = attr.ib(factory=list)
    outer_most_node = attr.ib(default=None)

    def write(self, seq_nodes: List[Node]):
        """
        Get textual representation of the sequence.
        Find for each named child of source_node, can we replace it with its respective target group.
        """
        node_queries = [self.write_query(node) for node in seq_nodes]

        self.query_str = ".".join(node_queries) + "\n" + "\n".join(self.query_ctrs)
        self.query_str = f"({self.query_str})"

        return self.query_str

    def write_query(self, node: Node, depth=0, prefix=""):
        """
        Write a query for a given node, considering its depth and prefix.
        """
        indent = " " * depth
        cursor: TreeCursor = node.walk()
        s_exp = indent + f"{prefix}({node.type} "

        next_child = cursor.goto_first_child()
        while next_child:
            child_node: Node = cursor.node
            if child_node.is_named:
                s_exp += "\n"
                prefix = (
                    f"{cursor.current_field_name()}: "
                    if cursor.current_field_name()
                    else ""
                )
                s_exp += self.write_query(cursor.node, depth + 1, prefix)
            next_child = cursor.goto_next_sibling()

        self.count += 1
        self.capture_groups[f"@tag{self.count}"] = node

        # if the node is an identifier, add it to eq constraints
        if node.child_count == 0:
            self.query_ctrs.append(
                f"(#eq? @tag{self.count} \"{node.text.decode('utf8')}\")"
            )

        self.outer_most_node = f"@tag{self.count}"
        return s_exp + f") @tag{self.count}"

    def replace_with_tags(self, replace_str: str) -> str:
        for capture_group, node in sorted(
            self.capture_groups.items(), key=lambda x: -len(x[1].text)
        ):
            text_repr = NodeUtils.convert_to_source(node)
            if text_repr in replace_str:
                replace_str = replace_str.replace(text_repr, f"{capture_group}")
        return replace_str


@attr.s
class Inference:
    lines_before = attr.ib(type=Dict[str, Node])
    lines_after = attr.ib(type=Dict[str, Node])
    _nodes_before = attr.ib(type=List[Node], init=False)
    _nodes_after = attr.ib(type=List[Node], init=False)

    """
    This class holds the functions for inferring and creating rules for node replacements
    """

    def __attrs_post_init__(self):
        self._nodes_before = NodeUtils.get_smallest_nonoverlapping_set(
            list(self.lines_before.values())
        )
        self._nodes_after = NodeUtils.get_smallest_nonoverlapping_set(
            list(self.lines_after.values())
        )

    def static_infer(self) -> List[str]:
        rules = []
        mappings = self.find_mappings()

        if len(self._nodes_after) > 0 and len(self._nodes_before) > 0:
            for node_id, replacements in mappings.items():
                node_it = filter(lambda x: x.id == node_id, self._nodes_before)
                rules += [self.create_replacement(next(node_it), replacements)]
        elif len(self._nodes_after) > 0:
            # We don't support additions for now. TODO
            pass
        elif len(self._nodes_before) > 0:
            for node_id, _ in mappings.items():
                node_it = filter(lambda x: x.id == node_id, self._nodes_before)
                rules += [self.create_deletion(next(node_it))]
        return rules

    def find_mappings(self) -> Dict[int, List[Node]]:
        """
        Basic mapping. Matches nodes in an orderly manner.
        """

        # compute mapping by search for isomorphic nodes
        mapping = {
            before.id: [after]
            for before, after in zip(self._nodes_before, self._nodes_after)
        }

        if len(self._nodes_before) > len(self._nodes_after):
            for node_before in self._nodes_before[len(self._nodes_after) :]:
                mapping[node_before.id] = []

        if len(self._nodes_before) < len(self._nodes_after):
            for node_after in self._nodes_after[len(self._nodes_before) :]:
                mapping[self._nodes_before[-1].id].append(node_after)

        return mapping

    def find_nodes_to_change(self, node_before: Node, node_after: Node):
        """
        Function to find nodes to change.
        """
        diverging_nodes = []
        if node_before.type == node_after.type:
            # Check if there's only one and only one diverging node
            # If there's more than one, then we can't do anything
            diverging_nodes = [
                (child_before, child_after)
                for child_before, child_after in zip(
                    node_before.named_children, node_after.named_children
                )
                if NodeUtils.convert_to_source(child_before)
                != NodeUtils.convert_to_source(child_after)
            ]

        if len(diverging_nodes) == 1:
            return self.find_nodes_to_change(*diverging_nodes[0])

        return node_before, [node_after]

    def create_replacement(self, node_before: Node, node_afters: List[Node]) -> str:
        """
        Create a rule based on the node before and after.
        """
        # For replacements (---- +++++)
        if len(node_afters) == 1:
            # Find whether we need to replace the entire node or just a part of it
            node_before, node_afters = self.find_nodes_to_change(
                node_before, node_afters[0]
            )

        qw = QueryWriter()
        query = qw.write([node_before])

        replace_str = "\\n".join(
            [NodeUtils.convert_to_source(node_after) for node_after in node_afters]
        )

        # Prioritize the longest strings first
        replace_str = qw.replace_with_tags(replace_str)

        rule = f'''[[rules]]\n\nquery = """{query}"""\n\nreplace_node = "{qw.outer_most_node[1:]}"\n\nreplace = "{replace_str}"'''

        # Check if the outermost node is in the replacement string
        # If so then we need to add a not_contains filter to prevent infinite recursion
        if qw.outer_most_node in replace_str:
            # Idea for a filter. The parent of node_before should not contain the outer_most_node
            # This is not a perfect filter, but it should work for most cases
            enclosing_node = f"({node_before.parent.type}) @parent"
            qw = QueryWriter(count=qw.count + 1)
            query = qw.write([node_afters[0]])
            not_contains = f"{query}"
            rule += f'''\n\n[[rules.filters]]\n\nenclosing_node = "{enclosing_node}"\n\nnot_contains = [\n"""{not_contains}\n"""]'''

        return rule

    def create_deletion(self, node_before: Node) -> str:
        """
        Simply delete the node
        """
        qw = QueryWriter()
        query = qw.write([node_before])

        # we have to check if we're deleting more than the actual line to be deleted.
        content_to_delete = NodeUtils.normalize_code("".join(self.lines_before.keys()))
        source = NodeUtils.normalize_code(NodeUtils.convert_to_source(node_before))

        # if content_to_delete not in source, replace is source (all content is kept)
        # if content_to_delete in source, replace is the remaining part after removing content_to_delete
        replace_str = source.replace(content_to_delete, "")

        # Prioritize the longest strings first
        replace_str = qw.replace_with_tags(replace_str)

        rule = f'''[[rules]]\n\nquery = """{query}"""\n\nreplace_node = "{qw.outer_most_node[1:]}"\n\nreplace = "{replace_str}"'''

        return rule

    def create_addition(self, anchor_node, node_after):
        pass
