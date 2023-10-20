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



        var conf2 = new SparkConf();
        conf2.set("spark.driver.instances:", "100");
        conf2.setAppName(appName);
        conf2.setSparkHome(sparkHome);

        sc2 = new JavaSparkContext(conf2);

       
    }
}
