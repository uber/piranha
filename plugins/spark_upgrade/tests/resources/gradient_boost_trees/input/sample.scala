package org.piranha

object GradientBoostTressExample {
    def main(args: Array[String]): Unit = {
    val (a, b) =
      GradientBoostedTrees.run(
        oldDataset,
        boostingStrategy,
        $(seed),
        "auto" /* featureSubsetStrategy */ )
  }

    val (x, y) =
      GradientBoostedTrees.run(
        another_dataset,
        boostingStrategy,
        $(seed),
        "auto" /* featureSubsetStrategy */ )
  }
}
