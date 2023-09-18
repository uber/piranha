# `Spark Upgrade` Plugin (WIP)

Upgrades your codebase to Spark 3.3


Currently, it updates to [v.3.3](https://spark.apache.org/releases/spark-release-3-3-0.html) only.
Supported rewrites: 
* `CalendarInterval` -> `DateTimeConstants`
* Updates execution plan access
* `Calculator` signature change
* `GradientBoostTrees` api change
* Update `SQLExecution.withNewExecutionId`
* Update `QueryTest.CheckAnswer`
* Update `SparkConf` (Builder pattern style)



## Usage: 

Clone the repository - `git clone https://github.com/uber/piranha.git`

Install the dependencies - `pip3 install -r plugins/spark_upgrade/requirements.txt`

Run the tool - `python3 plugins/spark_upgrade/main.py -h`

CLI: 
```
usage: main.py [-h] --path_to_codebase PATH_TO_CODEBASE

Updates the codebase to use a new version of `Spark 3.3`.

options:
  -h, --help            show this help message and exit
  --path_to_codebase PATH_TO_CODEBASE
                        Path to the codebase directory.
```

## Test
```
pytest plugins/
```
