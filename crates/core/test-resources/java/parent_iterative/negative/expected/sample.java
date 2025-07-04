package com.piranha;

import org.apache.spark.SparkConf;
import org.apache.spark.sql.SparkSession;

class Sample {
  void main() {
    // `setApp` and `setMaster` will not get updated
    val conf1 = new SparkSession.builder()
        .setMaster(master1)
        .setAppName(appName1)
        .executorEnv("spark.executor.extraClassPath1", "test1");

    // `setApp` and `setMaster` will not get updated
    val conf2 = new SparkSession.builder()
        .setAppName(appName2)
        .setMaster(master2)
        .sparkHome(sparkHome2)
        .executorEnv("spark.executor.extraClassPath2", "test2");

  // `setMaster` and `setExecutorEnv` will not get updated
    val conf3 = new SparkSession.builder()
        .setExecutorEnv("spark.executor.extraClassPath3", "test3")
        .setMaster(master3)
        .sparkHome(sparkHome3)
        .appName(appName3);
  }
}
