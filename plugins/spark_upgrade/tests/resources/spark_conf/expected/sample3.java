package com.piranha;

import org.apache.spark.SparkConf;
import org.apache.spark.SparkContext;
import org.apache.spark.sql.SparkSession;

public class Sample {
    public static void main(String[] args) {
        // Should not touch existent SparkSession.builder()
        SparkSession session = SparkSession.builder().config(sc.getConf()).getOrCreate();

        SparkSession session2 =
        SparkSession.builder().appName("appName").config("config", "local").getOrCreate();

        SparkSession.builder().appName("appName").config("config", "local").getOrCreate();
    }

    @Test
    public void test() {
        SparkSession session =
            SparkSession.builder().appName("appName").config("config", "local").getOrCreate();
        SparkContext sc = session.sparkContext();
        JavaSparkContext jsc = JavaSparkContext.fromSparkContext(sc());
    }
}
