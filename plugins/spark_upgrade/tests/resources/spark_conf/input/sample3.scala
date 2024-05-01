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
                .getOrCreate())
        spark.sqlContext.setConf("spark.ui.enabled", "false")
        spark.sqlContext.setConf("spark.driver.host", "localhost")
    }

}
