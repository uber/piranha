import org.apache.spark.SparkConf;

public class Sample2 {

  private static JavaSparkContext jsc;

  @BeforeClass
  public static void startSpark() {
    jsc =
        new JavaSparkContext(
            new SparkConf()
                .setAppName(Sample2.class.getName())
                .setMaster("master")
                .set("spark.driver.allowMultipleContexts", "true"));
  }

}
