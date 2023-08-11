# Copyright (c) 2023 Uber Technologies, Inc.
#
# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0
#
# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

from setuptools import find_packages, setup

setup(
    name="scala_test",
    version="0.0.1",
    description="Rules to migrate `scaletest`",
    # long_description=open("README.md").read(),
    # long_description_content_type="text/markdown",
    # url="https://github.com/uber/piranha",
    packages=find_packages(),
    include_package_data=True,
    install_requires=[
        # "polyglot-piranha",
        "pytest",
    ],
    entry_points={
        "console_scripts": ["scala_test = scala_test.main:main"]
    },
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.9",
    tests_require=["pytest"],
    # Define the test suite
    test_suite="tests",
)
