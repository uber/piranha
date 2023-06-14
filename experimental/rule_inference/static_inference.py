from tree_sitter import Node, TreeCursor
from typing import List
from patch import Patch
import attr


@attr.s
class QueryWriter:
    capture_groups = attr.ib(factory=dict)
    count = attr.ib(default=0)
    query_str = attr.ib(default="")
    query_ctrs = attr.ib(factory=list)
    outer_most_node = attr.ib(default=None)

    def write(self, seq_nodes: List[Node]):
        """
        Get textual representation of the sequence.
        Then find for each named child of source_node, can we replace it with its respective target group.
        """
        node_queries = []
        for node in seq_nodes:
            node_queries.append(self.write_query(node))

        self.query_str = ".".join(node_queries)
        self.query_str += "\n"
        self.query_str += "\n".join(self.query_ctrs)
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
                prefix = ""
                if cursor.current_field_name():
                    prefix = f"{cursor.current_field_name()}: "
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


def find_mappings(diff_hunk: str):
    """
    Use comments to come up with a mapping from source to target nodes.
    The nodes cannot be overlapping.
    Given a diff hunk, find exactly which source nodes were changed, select those.
    Find what they are replaced with.
    For example:
        (single edits)
        x.doSomething().else(someMethod()) ------> x.doSomething().else(Wrap(someMethod()), someMethod()))
        Change = someMethod() -----> replaced by Wrap(someMethod()), someMethod())

        (multiple edits two cases):
        same edit: Change   (e.g. call(someMethod(), someMethod()) ---> call(Wrap(someMethod()), Wrap(someMethod())))
        different edits: List[Change]   (write individual query for each, and then combine with the smallest common ancestor)
    """
    pass


# given a mapping , create rule always works
# all targets connect to a source
# the idea is then replace the source with targets if they are different,
# then you will always end up with the correct result, right?
def create_rule(node_before: Node, node_afters):
    """
    1. If it's an insertion.
        i. create a rule that inserts with a not contains filter to prevent recursive application
    2. If it's a deletion.
        i. simply create the match for the node with all eqs and delete the outer_most
    3. if it's a replacement:
        i. create a matcher for the nodes to be replaced
        ii. get a textual representation of the target nodes (unparse)
        iii. for each capture group in the matcher, check if it's possible to replace any substring of the target with the capture group
        iv. make sure the outermost left node is not the in replacement string, otherwise ifninite recursion. prevent this with a not_contains filter
    """
    qw = QueryWriter()
    query = qw.write([node_before])

    replace_str = "\\n".join([to_source(node_after) for node_after in node_afters])

    # Prioritize the longest strings first
    for capture_group, node in sorted(
        qw.capture_groups.items(), key=lambda x: -len(x[1].text)
    ):
        text_repr = to_source(node)
        if text_repr in replace_str:
            replace_str = replace_str.replace(text_repr, f"{capture_group}")

    print(query)
    print(qw.outer_most_node)
    print(replace_str)
