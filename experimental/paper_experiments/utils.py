
from tree_sitter import Language, Node, Parser, Tree
from tree_sitter_languages import get_parser

JAVA = "java"
SCALA = "scala"


JAVA_SOURCE_CODE = """package com.piranha;

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
        
         sc1 = new JavaSparkContext(conf1);
        


        var conf2 = new SparkConf();
        conf2.set("spark.driver.instances:", "100");
        conf2.setAppName(appName);
        conf2.setSparkHome(sparkHome);

        sc2 = new JavaSparkContext(conf2);

       
    }
}
"""



SCALA_SOURCE_CODE = """package com.piranha

import org.apache.spark.SparkConf
import org.apache.spark.sql.SparkSession

class Sample {
  def main(args: Array[String]): Unit = {
    
    val conf= new SparkConf()
      .setAppName("Sample App")
    
    val sc = new SparkContext(conf)

    val conf1 = new SparkConf()
      .setMaster(master)
      .setAll(Seq(("k2", "v2"), ("k3", "v3")))
      .setAppName(appName)
      .setSparkHome(sparkHome)
      .setExecutorEnv("spark.executor.extraClassPath", "test")
      .set("spark.driver.allowMultipleContexts", "true")
    sc1 = new SparkContext(conf1)
    

    val conf2 = new SparkConf()
      .setMaster(master)
    
    conf2.setSparkHome(sparkHome)

    conf2.setExecutorEnv("spark.executor.extraClassPath", "test")



  }

}
"""


Language.build_library(
  # Store the library in the `build` directory
  'build/my-languages.so',

  # Include one or more languages
  [
    '/Users/ketkara/repositories/open-source/tree-sitter-scala',
    '/Users/ketkara/repositories/open-source/tree-sitter-java',
  ]
)

SCALA_LANGUAGE = Language('build/my-languages.so', 'scala')
JAVA_LANGUAGE = Language('build/my-languages.so', 'java')

def parse_code(language: str, source_code: str) -> Tree:
    "Helper function to parse into tree sitter nodes"
    parser = Parser()
    parser.set_language(JAVA_LANGUAGE if language == JAVA else SCALA_LANGUAGE)

    source_tree = parser.parse(bytes(source_code, "utf8"))
    return source_tree

def traverse_tree(tree: Tree):
    cursor = tree.walk()

    reached_root = False
    while reached_root == False:
        yield cursor.node

        if cursor.goto_first_child():
            continue

        if cursor.goto_next_sibling():
            continue

        retracing = True
        while retracing:
            if not cursor.goto_parent():
                retracing = False
                reached_root = True

            if cursor.goto_next_sibling():
                retracing = False


def rewrite(node: Node, source_code: str, replacement: str, language: str):
    new_source_code = (
        source_code[: node.start_byte]
        + replacement
        + source_code[node.end_byte :]
    )
    print(new_source_code)
    return parse_code(language, new_source_code), new_source_code
