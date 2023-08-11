from recipes import replace_imports


IMPORT_MAPPING = {
    "org.scalatest.Matchers": "org.scalatest.matchers.should.Matcher",
    "org.scalatest.mock.MockitoSugar": "org.scalatestplus.mockito.MockitoSugar",
    # Todo write test scenarios for these
    "org.scalatest.FunSuite":"org.scalatest.funsuite.AnyFunSuite",
    "org.scalatest.junit.JUnitRunner":"org.scalatestplus.junit.JUnitRunner",
    "org.scalatest.FlatSpec": "org.scalatest.flatspec.AnyFlatSpec",
    "org.scalatest.junit.AssertionsForJUnit": "org.scalatestplus.junit.AssertionsForJUnit",
}

def update_imports(path_to_codebase: str, dry_run = False):
    return replace_imports(IMPORT_MAPPING, "scalatest", path_to_codebase, dry_run)
