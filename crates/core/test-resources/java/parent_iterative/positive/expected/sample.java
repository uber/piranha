package com.piranha;

import org.apache.spark.SparkConf;
import org.apache.spark.sql.SparkSession;

class Sample {
  void main() {

    val conf1 = new SparkSession.builder()
        .master(master1)
        .appName(appName1)
        .executorEnv("spark.executor.extraClassPath1", "test1");

    val conf2 = new SparkSession.builder()
      .appName(appName2)
      .master(master2)
      .sparkHome(sparkHome2)
      .executorEnv("spark.executor.extraClassPath2", "test2");

    val conf3 = new SparkSession.builder()
      .executorEnv("spark.executor.extraClassPath3", "test3")
      .appName(appName3)
      .master(master3)
      .sparkHome(sparkHome3);
  }
}
