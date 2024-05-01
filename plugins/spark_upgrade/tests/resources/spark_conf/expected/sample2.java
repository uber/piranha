import org.apache.spark.SparkConf;
import org.apache.spark.sql.SparkSession;

public class Sample2 {

  private static JavaSparkContext jsc;

  @BeforeClass
  public static void startSpark() {
    jsc =
        new JavaSparkContext(
            SparkSession.builder()
                .config("spark.sql.legacy.allowUntypedScalaUDF", "true")
                .appName(Sample2.class.getName())
                .master("master")
                .config("spark.driver.allowMultipleContexts", "true")
                .getOrCreate()
                .sparkContext());
  }

}
