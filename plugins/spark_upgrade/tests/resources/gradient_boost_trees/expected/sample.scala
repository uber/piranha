package org.piranha

object GradientBoostTressExample {
  def main(args: Array[String]): Unit = {
    val (baseLearners: Array[DecisionTreeRegressionModel], learnerWeights) =
      GradientBoostedTrees.run(
        oldDataset.map(data => new Instance(data.label, 1.0, data.features)),
        boostingStrategy,
        $(seed),
        "auto" /* featureSubsetStrategy */ )
  }
}
