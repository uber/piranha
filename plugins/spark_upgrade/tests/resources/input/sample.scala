package org.piranha

import org.apache.spark.unsafe.types.CalendarInterval

object CalendarIntervalExample {
  def main(args: Array[String]): Unit = {
    // Accessing MICROS_PER_SECOND constant
    val microsPerSecond = CalendarInterval.MICROS_PER_SECOND
    val microsPerHour = CalendarInterval.MICROS_PER_HOUR
    val fromYearMonthString = CalendarInterval.fromYearMonthString("1-2")
    val fromDayTimeString = CalendarInterval.fromDayTimeString("1-2")
    println(s"Microseconds per Second: $microsPerSecond")
  }
}
