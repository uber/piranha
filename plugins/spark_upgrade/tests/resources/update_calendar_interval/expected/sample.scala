package org.piranha

import org.apache.spark.sql.catalyst.util.DateTimeConstants
import org.apache.spark.unsafe.types.CalendarInterval

object CalendarIntervalExample {
  def main(args: Array[String]): Unit = {
    // Accessing MICROS_PER_SECOND constant
    val microsPerSecond = DateTimeConstants.MICROS_PER_SECOND
    val microsPerHour = DateTimeConstants.MICROS_PER_HOUR
    val fromYearMonthString = DateTimeConstants.fromYearMonthString("1-2")
    val fromDayTimeString = DateTimeConstants.fromDayTimeString("1-2")
    println(s"Microseconds per Second: $microsPerSecond")
  }
}
