#  Copyright (c) 2023 Uber Technologies, Inc.

#  <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
#  except in compliance with the License. You may obtain a copy of the License at
#  <p>http://www.apache.org/licenses/LICENSE-2.0

#  <p>Unless required by applicable law or agreed to in writing, software distributed under the
#  License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
#  express or implied. See the License for the specific language governing permissions and
#  limitations under the License.
 
import argparse
import logging

from update_calendar_interval import UpdateCalendarInterval
from IDF_model_signature_change import IDFModelSignatureChange
from accessing_execution_plan import AccessingExecutionPlan
from gradient_boost_trees import GradientBoostTrees
from calculator_signature_change import CalculatorSignatureChange

def _parse_args():
    parser = argparse.ArgumentParser(
        description="Updates the codebase to use a new version of `spark3`"
    )
    parser.add_argument(
        "--path_to_codebase",
        required=True,
        help="Path to the codebase directory.",
    )
    parser.add_argument(
        "--new_version",
        default="3.3",
        help="Version of `Spark` to update to.",
    )
    args = parser.parse_args()
    return args


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)

def main():
    args = _parse_args()
    if args.new_version == "3.3":
        upgrade_to_spark_3_3(args.path_to_codebase)


def upgrade_to_spark_3_3(path_to_codebase):
    update_calendar_interval = UpdateCalendarInterval([path_to_codebase])
    _ = update_calendar_interval()
    
    idf_model_signature_change = IDFModelSignatureChange([path_to_codebase])
    _ = idf_model_signature_change()
    
    accessing_execution_plan = AccessingExecutionPlan([path_to_codebase])
    _ = accessing_execution_plan()

    gradient_boost_trees = GradientBoostTrees([path_to_codebase])
    _ = gradient_boost_trees()
    
    calculator_signature_change = CalculatorSignatureChange([path_to_codebase])
    _ = calculator_signature_change()
    
if __name__ == "__main__":
    main()
