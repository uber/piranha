package com.uber.piranha;

/**
 * An exception thrown during the execution of the Piranha bug checker. This is a runtime exception
 * and will crash piranha, as the tool has detected that it will clean up code in an invalid state.
 */
public final class PiranhaRuntimeException extends RuntimeException {

  public PiranhaRuntimeException(String message) {
    super(message);
  }
}
