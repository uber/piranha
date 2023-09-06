package org.piranha

object IDFModelExample {
  def main(args: Array[String]): Unit = {
    var r = new feature.IDFModel(
        MichelangeloIDFModel.getVectorFromProto(protoTransformer.getIDF.getIdfModel),
        new Array[Long](protoTransformer.getTopicSimilarity.getIdfModel.getIdfModel.getSize),
        0L)
  }
}
