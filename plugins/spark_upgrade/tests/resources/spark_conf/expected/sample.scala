package com.piranha;

import org.apache.spark.SparkConf
import org.apache.spark.sql.SparkSession

class Sample {
  def main(args: Array[String]): Unit = {

    val conf = new SparkConf()
      .set("spark.sql.legacy.timeParserPolicy","LEGACY")
      .set("spark.sql.legacy.allowUntypedScalaUDF", "true")
      .setMaster(master)
      .setAppName(appName)
      .set("spark.driver.allowMultipleContexts", "true")
    val sc = new SparkContext(conf)
    val sqlContext = new TestHiveContext(sc).sparkSession

    val conf2 = new SparkConf()
      .set("spark.sql.legacy.timeParserPolicy","LEGACY")
      .set("spark.sql.legacy.allowUntypedScalaUDF", "true")

    conf2.setSparkHome(sparkHome)

    conf2.setExecutorEnv("spark.executor.extraClassPath", "test")

    val sparkSession = SparkSession.builder()
        .master(master)
        .appName(appName)
        .getOrCreate

  }

}
