from toml.encoder import TomlEncoder, _dump_str, unicode


def _dump_str_prefer_multiline(v):
    multilines = v.split("\n")
    if len(multilines) > 1:
        return unicode('"""\n' + v.replace('"""', '\\"""').strip() + '\n"""')
    else:
        return _dump_str(v)


class PrettyTOML(TomlEncoder):
    def __init__(self, _dict=dict, preserve=False):
        super(PrettyTOML, self).__init__(_dict=dict, preserve=preserve)
        self.dump_funcs[str] = _dump_str_prefer_multiline
