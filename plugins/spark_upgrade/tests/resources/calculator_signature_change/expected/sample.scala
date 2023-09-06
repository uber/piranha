package org.piranha

object AccessingExecutionPlan {
  def main(args: Array[String]): Unit = {
    var e1 = EntropyCalculator(s, s.sum.toLong)
    var e2 = GiniCalculator(s, s.sum.toLong)
    var e3 = VarianceCalculator(s, s.sum.toLong)
  }
}
