package org.piranha

import org.apache.spark.unsafe.types.CalendarInterval

object SqlNewExecutionExample {
  def main(args: Array[String]): Unit = {
    val x = SQLExecution.withNewExecutionId(sparkSession, qe)
  }
}
