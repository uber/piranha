package org.piranha

object AccessingExecutionPlan {
  def main(args: Array[String]): Unit = {
    var r0 = df.queryExecution.executedPlan.asInstanceOf[AdaptiveSparkPlanExec].initialPlan
    var r1 = df.queryExecution.executedPlan.asInstanceOf[AdaptiveSparkPlanExec].initialPlan.collect
  }
}
