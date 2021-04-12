package com.uber.piranha.testannotations;

import com.sun.source.tree.ExpressionTree;
import javax.annotation.Nullable;

/**
 * An object representing an argument passed to a java annotation usage.
 *
 * <p>This is used in the canonical representation of experiment flag test annotations defined by
 * {@link ResolvedTestAnnotation}.
 *
 * <p>This object contains both the value being passed (in string representation) and a reference to
 * the AST node representing the argument. In cases where an annotation field is assigned multiple
 * values, such as when an array expression is passed, one AnnotationArgument record is created per
 * value
 *
 * <p>For example, for annotation {@code @TestAnnotation(flags={FLAG_1, FLAG_2})}, two objects
 * should be created, one containing value {@code "FLAG_1"} and a pointer to the AST representing
 * the {@code FLAG_1} identifier in that expression, and an analogous but independent one for {@code
 * FLAG_2}.
 *
 * <p>When used as part of {@link ResolvedTestAnnotation}, some argument records might represent a
 * synthetic argument, without a corresponding AST node. In that case, the source AST node component
 * is {@code null}. An example would be the {@link ResolvedTestAnnotation} for
 * {@code @NewToggleControl("flag1")}, where the canonical representation needs an argument
 * representing the {@code treated} component of the test annotation (i.e. whether we are asking for
 * the flag in treatment or control condition for the annotated test). However, this is always just
 * the literal {@code "false"} for {@code NewToggleControl} (see Piranha's example {@code
 * properties.json})
 */
public final class AnnotationArgument {
  private final String value;
  @Nullable private final ExpressionTree sourceTree;

  /**
   * Create a new {@link AnnotationArgument} record with the given value and AST reference.
   *
   * @param value The annotation argument's value.
   * @param sourceTree The (optional) reference to this annotation argument in the AST (for
   *     potential deletion).
   */
  public AnnotationArgument(String value, @Nullable ExpressionTree sourceTree) {
    this.value = value;
    this.sourceTree = sourceTree;
  }

  /**
   * Return the annotation argument's value.
   *
   * @return The annotation argument's value.
   */
  public String getValue() {
    return value;
  }

  /**
   * Return the reference to this annotation argument in the AST (if any).
   *
   * @return An AST node. See {@link AnnotationArgument} class-level docs.
   */
  @Nullable
  public ExpressionTree getSourceTree() {
    return sourceTree;
  }
}
