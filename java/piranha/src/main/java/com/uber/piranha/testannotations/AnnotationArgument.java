package com.uber.piranha.testannotations;

import com.sun.source.tree.ExpressionTree;
import javax.annotation.Nullable;

public final class AnnotationArgument {
  private final String value;
  @Nullable private final ExpressionTree sourceTree;

  public AnnotationArgument(String value, @Nullable ExpressionTree sourceTree) {
    this.value = value;
    this.sourceTree = sourceTree;
  }

  public String getValue() {
    return value;
  }

  @Nullable
  public ExpressionTree getSourceTree() {
    return sourceTree;
  }
}
