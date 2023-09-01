package org.piranha

object AccessingExecutionPlan {
  def main(args: Array[String]): Unit = {
    var r = df.queryExecution.executedPlan.collect
  }
}
