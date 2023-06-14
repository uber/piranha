from tree_sitter import TreeCursor, Node
import attr


class NodeWriter:
    @staticmethod
    def generate_sexpr(node, depth=0, prefix=""):
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
                s_exp += NodeWriter.generate_sexpr(child_node, depth + 1, prefix)
            next_child = cursor.goto_next_sibling()
        return s_exp + ")"

    @staticmethod
    def convert_to_source(node, depth=0):
        cursor: TreeCursor = node.walk()
        s_exp = ""
        next_child = cursor.goto_first_child()
        if not next_child:
            s_exp += node.text.decode("utf8")
            return s_exp

        while next_child:
            s_exp += NodeWriter.convert_to_source(next_child, depth + 1)
            next_child = cursor.goto_next_sibling()
        return s_exp
