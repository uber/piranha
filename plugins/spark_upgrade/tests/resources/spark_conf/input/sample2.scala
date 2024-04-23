package com.piranha;

import org.apache.spark.SparkConf
import org.apache.spark.sql.SparkSession

class Sample {

  def main(argv: Array[String]): Unit = {
        val spark = SparkSession.builder
        .appName("appName")
        .config("conf", "package.conf")
        .enableHiveSupport
        .getOrCreate

        run(spark, config)
  }

}
