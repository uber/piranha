# `scalatest` Migration Plugin (WIP)

This piranha plugin updates `scalatest` to a new version.


Currently, it updates to [v.3.2.2](https://mvnrepository.com/artifact/org.scalatest/scalatest_2.12/3.2.2) only. The following import statements are updated: 
* `org.scalatest.Matchers`-> `org.scalatest.matchers.should.Matchers`
* `org.scalatest.mock.MockitoSugar`-> `org.scalatestplus.mockito.MockitoSugar`
* `org.scalatest.FunSuite`->`org.scalatest.funsuite.AnyFunSuite`
* `org.scalatest.junit.JUnitRunner`->`org.scalatestplus.junit.JUnitRunner`
* `org.scalatest.FlatSpec`-> `org.scalatest.flatspec.AnyFlatSpec`
* `org.scalatest.junit.AssertionsForJUnit`-> `org.scalatestplus.junit.AssertionsForJUnit`

## Usage: 

Clone the repository - `git clone https://github.com/uber/piranha.git`

Install the dependencies - `pip3 install -r plugins/scala_test/requirements.txt`

Run the tool - `python3 plugins/scala_test/main.py -h`

CLI: 
```
usage: main.py [-h] --path_to_codebase PATH_TO_CODEBASE

Updates the codebase to use a new version of `scalatest_2.12`.

options:
  -h, --help            show this help message and exit
  --path_to_codebase PATH_TO_CODEBASE
                        Path to the codebase directory.
```

## Test
```
pytest plugins/scala_test
```
