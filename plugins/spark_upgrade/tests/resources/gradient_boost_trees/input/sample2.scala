package org.piranha

object GradientBoostTressExample {
    def main(args: Array[String]): Unit = {
    val (c, d) =
      GradientBoostedTrees.run(
        oldDataset,
        boostingStrategy,
        seed,
        "auto")
  }
}
