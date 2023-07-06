import attr
import toml
from experimental.rule_inference.utils.pretty_toml import PrettyTOML
from experimental.rule_inference.utils.rule_utils import RawRuleGraph


@attr.s
class InferData:
    """
    Data class representing the information needed for the infer_piranha event.
    """

    source_code = attr.ib(validator=attr.validators.instance_of(str))
    target_code = attr.ib(validator=attr.validators.instance_of(str))
    language = attr.ib(validator=attr.validators.in_(["kt", "java"]))


@attr.s
class ImproveData:
    """
    Data class representing the information needed for the improve_piranha event.
    """

    language = attr.ib(validator=attr.validators.in_(["kt", "java"]))
    requirements = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=RawRuleGraph.validate)

    def __attrs_post_init__(self):
        self.rules = toml.dumps(toml.loads(self.rules), encoder=PrettyTOML())


@attr.s
class RefactorData:
    """
    Data class representing the information needed for the refactor_codebase event.
    """

    language = attr.ib(validator=attr.validators.in_(["kt", "java"]))
    folder_path = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=RawRuleGraph.validate)


@attr.s
class RefactorSnippet:
    """
    Data class representing the information needed for the refactor_codebase event.
    """

    language = attr.ib(validator=attr.validators.in_(["kt", "java"]))
    source_code = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=RawRuleGraph.validate)
