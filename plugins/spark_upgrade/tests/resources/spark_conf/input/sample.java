package com.piranha;

import org.apache.spark.SparkConf;
import org.apache.spark.SparkContext;
import org.apache.spark.sql.SparkSession;
import org.spark_project.jetty.util.ArrayQueue;

public class Sample {
    public static void main(String[] args) {
        String master = "local";
        String appName = "SampleApp";
        String sparkHome = "your_spark_home_directory";

        SparkConf conf = new SparkConf()
                .setMaster(master)
                .setAppName(appName)
                .set("spark.driver.allowMultipleContexts", "true");

        SparkContext sc = new SparkContext(conf);
        TestHiveContext sqlContext = new TestHiveContext(sc);
        SparkSession sparkSession = sqlContext.sparkSession();

        SparkConf conf2 = new SparkConf();
        conf2.setSparkHome(sparkHome);

        conf2.setExecutorEnv("spark.executor.extraClassPath", "test");

        // Should not touch existent SparkSession.builder()
        SparkSession sparkSession = SparkSession.builder()
                .master(master)
                .appName(appName)
                .getOrCreate();
    }
}
