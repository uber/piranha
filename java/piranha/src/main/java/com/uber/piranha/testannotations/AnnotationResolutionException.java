package com.uber.piranha.testannotations;

/**
 * An exception thrown when annotation resolution fails. This is a runtime exception and will crash
 * piranha. It indicates a configuration error, usually an unrecoverable mismatch between the shape
 * of the annotations specified in Piranha's json configuration file, and an annotation usage with
 * the same name in the code being rewritten.
 */
public class AnnotationResolutionException extends RuntimeException {

  public AnnotationResolutionException(String message) {
    super(message);
  }
}
