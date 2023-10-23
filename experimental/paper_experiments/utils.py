
from tree_sitter import Node, Tree
from tree_sitter_languages import get_parser


SOURCE_CODE = """package com.piranha;

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
        
         sc1 = new JavaSparkContext(conf1);
        


        var conf2 = new SparkConf();
        conf2.set("spark.driver.instances:", "100");
        conf2.setAppName(appName);
        conf2.setSparkHome(sparkHome);

        sc2 = new JavaSparkContext(conf2);

       
    }
}
"""


def parse_code(language: str, source_code: str) -> Tree:
    "Helper function to parse into tree sitter nodes"
    parser = get_parser(language)
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


def rewrite(node: Node, source_code: str, replacement: str):
    new_source_code = (
        source_code[: node.start_byte]
        + replacement
        + source_code[node.end_byte :]
    )
    print(new_source_code)
    return parse_code("java", new_source_code), new_source_code
