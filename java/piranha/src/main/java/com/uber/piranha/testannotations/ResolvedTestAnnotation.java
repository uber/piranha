package com.uber.piranha.testannotations;

import com.google.common.collect.ImmutableSet;
import com.sun.source.tree.AnnotationTree;

public final class ResolvedTestAnnotation {
  private final ImmutableSet<AnnotationArgument> flags;
  private final boolean treated;
  private final ImmutableSet<AnnotationArgument> groups;
  private final AnnotationTree sourceTree;

  public ResolvedTestAnnotation(
      ImmutableSet<AnnotationArgument> flags,
      boolean treated,
      ImmutableSet<AnnotationArgument> groups,
      AnnotationTree source) {
    this.flags = flags;
    this.treated = treated;
    this.groups = groups;
    this.sourceTree = source;
  }

  public ImmutableSet<AnnotationArgument> getFlags() {
    return flags;
  }

  public boolean isTreated() {
    return treated;
  }

  public ImmutableSet<AnnotationArgument> getGroups() {
    return groups;
  }

  public AnnotationTree getSourceTree() {
    return sourceTree;
  }
}
