package com.piranha;

import org.apache.spark.SparkConf
import org.apache.spark.sql.SparkSession

class Sample {
  def main(args: Array[String]): Unit = {
    
    val conf= new SparkSession.builder()
        .config("spark.sql.legacy.allowUntypedScalaUDF", "true")
        .appName("Sample App")
        .getOrCreate()

    val sc = conf.sparkContext


    val conf1 = new SparkSession.builder()
        .config("spark.sql.legacy.allowUntypedScalaUDF", "true")
        .master(master)
        .all(Seq(("k2", "v2"), ("k3", "v3")))
        .appName(appName)
        .sparkHome(sparkHome)
        .executorEnv("spark.executor.extraClassPath", "test")
        .config("spark.driver.allowMultipleContexts", "true")
        .getOrCreate()
    sc1 = conf1.sparkContext
    
    val conf2 = new SparkSession.builder()
        .config("spark.sql.legacy.allowUntypedScalaUDF", "true")
        .master(master)
        .getOrCreate()
    conf2.sparkHome(sparkHome)

    conf2.executorEnv("spark.executor.extraClassPath", "test")

  }
}
