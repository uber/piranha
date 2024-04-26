package com.piranha;

import org.apache.spark.sql.SparkSession

import org.mockito.Mockito.spy

class Sample {

    @Test
    def testMain(): Unit = {
        lazy val spark: SparkSession = spy(
            SparkSession
                .builder()
                .master("master")
                .appName("AppName")
                .config("spark.ui.enabled", "false")
                .config("spark.driver.host", "localhost")
                .getOrCreate())
    }

}
