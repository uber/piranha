package com.piranha;

import org.apache.spark.SparkConf
import org.apache.spark.sql.SparkSession

class Sample {
  def main(args: Array[String]): Unit = {
    
    val conf= new SparkConf()
      .setAppName("Sample App")
    
    val sc = new SparkContext(conf)

    val conf1 = new SparkConf()
      .setMaster(master)
      .setAll(Seq(("k2", "v2"), ("k3", "v3")))
      .setAppName(appName)
      .setSparkHome(sparkHome)
      .setExecutorEnv("spark.executor.extraClassPath", "test")
      .set("spark.driver.allowMultipleContexts", "true")
    sc1 = new SparkContext(conf1)
    
  }

}
