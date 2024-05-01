package org.piranha

// this rules removes the comment on line 11
object GradientBoostTressExample {
  def main(args: Array[String]): Unit = {
    val (a, b) =
      GradientBoostedTrees.run(
        oldDataset.map(data => new Instance(data.label, 1.0, data.features)),
        boostingStrategy,
        seed,
        "auto")
  }
}
