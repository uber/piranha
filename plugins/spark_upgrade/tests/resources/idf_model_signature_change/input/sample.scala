package org.piranha

object IDFModelExample {
  def main(args: Array[String]): Unit = {
    var r = feature.IDFModel(
      MichelangeloIDFModel.getVectorFromProto(protoTransformer.getIDF.getIdfModel)
    )
  }
}
