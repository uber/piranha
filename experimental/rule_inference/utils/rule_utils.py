import json
from typing import List, Dict
from polyglot_piranha import Rule, RuleGraph, OutgoingEdges


class RawRule:
    def __init__(self, *args, **kwargs):
        self.wrapped = Rule(*args, **kwargs)
        self.args = args
        self.kwargs = kwargs

    def __getattr__(self, name):
        return self.kwargs.get(name) or getattr(self.wrapped, name, None)

    def to_toml(self):
        str_reprs = [
            f"name={self.name}",
            f'query = """{self.query}"""',
            f"replace_node = {self.replace_node}",
            f'replace = """{self.replace}"""',
        ]
        return "[[rules]]\n" + "\n".join(str_reprs)


class RawRuleGraph:
    def __init__(self, rules: List[RawRule], edges: Dict[str, List[str]]):
        self.rules = rules
        self.edges = edges

    def to_toml(self):
        rules_str = "\n\n".join(rule.to_toml() for rule in self.rules)

        edges_str = "\n\n"
        if self.edges:
            edges_str = "\n\n".join(
                self.edge_to_toml(source, destinations)
                for source, destinations in self.edges.items()
            )

        return f"{rules_str}\n{edges_str}"

    @staticmethod
    def edge_to_toml(source, destinations):
        return "\n".join(
            [
                "[[edges]]",
                f'scope = "Global"',
                f'from = "{source}"',
                f"to = {json.dumps(destinations)}",
            ]
        )

    def get_graph(self):
        return RuleGraph(
            [rule.wrapped for rule in self.rules],
            [OutgoingEdges(*edge) for edge in self.edges.items()],
        )
