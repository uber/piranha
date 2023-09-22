package com.piranha;

import org.apache.spark.SparkConf;
import org.apache.spark.api.java.JavaSparkContext;

public class Sample {
    public static void main(String[] args) {
        SparkConf conf = new SparkConf()
                .setAppName("Sample App");

        JavaSparkContext sc = new JavaSparkContext(conf);

        SparkConf conf1 = new SparkConf()
          .setSparkHome(sparkHome)
          .setExecutorEnv("spark.executor.extraClassPath", "test")
          .setAppName(appName)
          .setMaster(master)
          .set("spark.driver.allowMultipleContexts", "true");
        
        sc = new JavaSparkContext(conf1);

       
    }
}
