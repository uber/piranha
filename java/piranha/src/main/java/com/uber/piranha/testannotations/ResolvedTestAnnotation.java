package com.uber.piranha.testannotations;

import com.google.common.collect.ImmutableList;
import com.sun.source.tree.AnnotationTree;

/**
 * Represents an experiment flag test annotation in a canonical form understood by Piranha's core.
 *
 * <p>This abstracts the actual shape (field names, types, default values) of the annotation in the
 * code (using the specifications in Piranha's json configuration file, see also {@link
 * TestAnnotationSpecRecord}) into a common record. An instance of a test annotation, in this
 * representation, consist of: a list of flags (possibly one), all sharing a treatment value
 * (treated or control, represented as a boolean), and possibly a list of treatment groups.
 *
 * <p>Additionally, each resolved annotation points to its original location in the AST.
 */
public final class ResolvedTestAnnotation {
  private final ImmutableList<AnnotationArgument> flags;
  private final boolean treated;
  private final ImmutableList<AnnotationArgument> groups;
  private final AnnotationTree sourceTree;

  // Constructor is package-private, as only TestAnnotationResolver should be constructing these
  // records.
  ResolvedTestAnnotation(
      ImmutableList<AnnotationArgument> flags,
      boolean treated,
      ImmutableList<AnnotationArgument> groups,
      AnnotationTree source) {
    this.flags = flags;
    this.treated = treated;
    this.groups = groups;
    this.sourceTree = source;
  }

  /**
   * Retrieve the list of flags referenced by this annotation.
   *
   * @return The list of flags referenced by this annotation.
   */
  public ImmutableList<AnnotationArgument> getFlags() {
    return flags;
  }

  /**
   * Retrieve the treatment condition requested by this annotation.
   *
   * @return {@code true} if the annotation is requesting the flags in {@code treated} condition,
   *     {@code false} for {@code control}.
   */
  public boolean isTreated() {
    return treated;
  }

  /**
   * Retrieve the treatment groups requested by this annotation.
   *
   * @return A list of treatment groups requested by this annotation, matching the {@link
   *     #getFlags()} positionally.
   */
  public ImmutableList<AnnotationArgument> getGroups() {
    return groups;
  }

  /**
   * Retrieve the AST node representing this annotation.
   *
   * @return the AST node representing this annotation in the analyzed source.
   */
  public AnnotationTree getSourceTree() {
    return sourceTree;
  }
}
